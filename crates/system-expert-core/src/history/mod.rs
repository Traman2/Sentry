//! Persistent history of [`super::monitor::SystemSnapshot`] samples, so the UI can
//! chart CPU/memory/disk-I/O trends over time instead of only ever seeing the live
//! snapshot.
//!
//! Recording is decoupled from any particular consumer polling the live snapshot:
//! [`spawn_recorder`] runs its own [`Monitor`] on a background thread and writes to
//! SQLite on a fixed interval, so history keeps accumulating even if nothing is
//! reading `Monitor::snapshot()` for display (e.g. no UI window open).

mod query;

pub use query::{ProcessSamplePoint, SystemSamplePoint};

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use crate::monitor::{Monitor, SystemSnapshot};

/// How long raw samples are kept before [`HistoryStore::prune_older_than`] deletes
/// them. [`spawn_recorder`] enforces this automatically once an hour.
pub const DEFAULT_RETENTION: Duration = Duration::from_secs(24 * 60 * 60);

/// A SQLite-backed store of historical system/process samples.
///
/// Safe to share across threads via `Arc<HistoryStore>`: all access goes through an
/// internal `Mutex<Connection>`, since a single `rusqlite::Connection` cannot be used
/// from multiple threads concurrently. Write-Ahead-Logging is enabled so the
/// underlying file also tolerates being opened by a second, independent connection
/// (e.g. a short-lived query-only connection) while this one holds it open.
pub struct HistoryStore {
    conn: Mutex<Connection>,
}

impl HistoryStore {
    /// Opens (creating if needed) a history database at `path` and ensures its schema
    /// exists.
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS system_samples (
                timestamp_ms                INTEGER PRIMARY KEY,
                cpu_usage_percent             REAL NOT NULL,
                used_memory_bytes               INTEGER NOT NULL,
                total_memory_bytes                INTEGER NOT NULL,
                used_swap_bytes                     INTEGER NOT NULL,
                network_rx_bytes_per_sec              INTEGER NOT NULL,
                network_tx_bytes_per_sec                INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS process_samples (
                id                          INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp_ms                  INTEGER NOT NULL,
                pid                             INTEGER NOT NULL,
                name                             TEXT NOT NULL,
                cpu_usage_percent                 REAL NOT NULL,
                memory_bytes                        INTEGER NOT NULL,
                disk_read_bytes_per_sec               INTEGER NOT NULL,
                disk_written_bytes_per_sec              INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_process_samples_pid_time
                ON process_samples(pid, timestamp_ms);
            CREATE INDEX IF NOT EXISTS idx_process_samples_time
                ON process_samples(timestamp_ms);
            ",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Opens an in-memory database. Useful for tests; not persisted to disk.
    pub fn open_in_memory() -> rusqlite::Result<Self> {
        Self::open(":memory:")
    }

    /// Records one row of system-wide aggregates and one row per process from
    /// `snapshot`, in a single transaction.
    pub fn record(&self, snapshot: &SystemSnapshot) -> rusqlite::Result<()> {
        let mut conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let tx = conn.transaction()?;

        let timestamp_ms = snapshot.timestamp_ms as i64;
        let network_rx_bytes_per_sec: i64 = snapshot
            .networks
            .iter()
            .map(|n| n.received_bytes_per_sec as i64)
            .sum();
        let network_tx_bytes_per_sec: i64 = snapshot
            .networks
            .iter()
            .map(|n| n.transmitted_bytes_per_sec as i64)
            .sum();

        tx.execute(
            "INSERT OR REPLACE INTO system_samples (
                timestamp_ms, cpu_usage_percent, used_memory_bytes, total_memory_bytes,
                used_swap_bytes, network_rx_bytes_per_sec, network_tx_bytes_per_sec
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                timestamp_ms,
                snapshot.system.global_cpu_usage_percent,
                snapshot.system.used_memory_bytes as i64,
                snapshot.system.total_memory_bytes as i64,
                snapshot.system.used_swap_bytes as i64,
                network_rx_bytes_per_sec,
                network_tx_bytes_per_sec,
            ],
        )?;

        {
            let mut insert_process = tx.prepare(
                "INSERT INTO process_samples (
                    timestamp_ms, pid, name, cpu_usage_percent, memory_bytes,
                    disk_read_bytes_per_sec, disk_written_bytes_per_sec
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            )?;
            for process in &snapshot.processes {
                insert_process.execute(rusqlite::params![
                    timestamp_ms,
                    process.pid as i64,
                    process.name,
                    process.cpu_usage_percent,
                    process.memory_bytes as i64,
                    process.disk_read_bytes_per_sec as i64,
                    process.disk_written_bytes_per_sec as i64,
                ])?;
            }
        }

        tx.commit()
    }

    /// Deletes samples older than `retention`, measured back from now.
    pub fn prune_older_than(&self, retention: Duration) -> rusqlite::Result<()> {
        let cutoff_ms = now_ms().saturating_sub(retention.as_millis() as i64);
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "DELETE FROM system_samples WHERE timestamp_ms < ?1",
            [cutoff_ms],
        )?;
        conn.execute(
            "DELETE FROM process_samples WHERE timestamp_ms < ?1",
            [cutoff_ms],
        )?;
        Ok(())
    }
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Signal handle returned by [`spawn_recorder`]; dropping it does not stop the
/// recorder (it runs for the process lifetime by default) — call [`RecorderHandle::stop`]
/// to end it explicitly (e.g. in tests).
pub struct RecorderHandle {
    stop_flag: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

impl RecorderHandle {
    /// Signals the recorder loop to stop and waits for it to exit.
    pub fn stop(mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Spawns a background thread that owns its own [`Monitor`] and, independent of
/// whatever else may or may not be polling `Monitor::snapshot()` for display, takes a
/// snapshot every `interval` and records it to `history`. Also prunes samples older
/// than [`DEFAULT_RETENTION`] once an hour.
///
/// The recorder runs for the lifetime of the process unless the returned
/// [`RecorderHandle`] is stopped explicitly.
pub fn spawn_recorder(history: Arc<HistoryStore>, interval: Duration) -> RecorderHandle {
    let stop_flag = Arc::new(AtomicBool::new(false));
    let thread_stop_flag = stop_flag.clone();

    let join_handle = thread::spawn(move || {
        let mut monitor = Monitor::new();
        let mut last_prune = SystemTime::now();
        let prune_interval = Duration::from_secs(60 * 60);

        while !thread_stop_flag.load(Ordering::Relaxed) {
            let snapshot = monitor.snapshot();
            if let Err(err) = history.record(&snapshot) {
                eprintln!("system-expert-core: failed to record history sample: {err}");
            }

            if last_prune.elapsed().unwrap_or_default() >= prune_interval {
                if let Err(err) = history.prune_older_than(DEFAULT_RETENTION) {
                    eprintln!("system-expert-core: failed to prune history: {err}");
                }
                last_prune = SystemTime::now();
            }

            thread::sleep(interval);
        }
    });

    RecorderHandle {
        stop_flag,
        join_handle: Some(join_handle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_then_query_round_trips() {
        let store = HistoryStore::open_in_memory().unwrap();
        let mut monitor = Monitor::new();
        let snapshot = monitor.snapshot();
        let some_pid = snapshot.processes.first().map(|p| p.pid);

        store.record(&snapshot).unwrap();

        let system_points = store.system_timeline(Duration::from_secs(60)).unwrap();
        assert_eq!(system_points.len(), 1);
        assert_eq!(system_points[0].timestamp_ms, snapshot.timestamp_ms as i64);

        if let Some(pid) = some_pid {
            let process_points = store
                .process_timeline(pid, Duration::from_secs(60))
                .unwrap();
            assert_eq!(process_points.len(), 1);
            assert_eq!(process_points[0].pid, pid);
        }
    }

    #[test]
    fn prune_removes_samples_older_than_retention() {
        let store = HistoryStore::open_in_memory().unwrap();
        let mut monitor = Monitor::new();
        store.record(&monitor.snapshot()).unwrap();

        thread::sleep(Duration::from_millis(5));
        store.prune_older_than(Duration::from_secs(0)).unwrap();

        let points = store.system_timeline(Duration::from_secs(60)).unwrap();
        assert!(points.is_empty());
    }

    #[test]
    fn query_excludes_samples_outside_the_requested_window() {
        let store = HistoryStore::open_in_memory().unwrap();
        let mut monitor = Monitor::new();
        store.record(&monitor.snapshot()).unwrap();

        thread::sleep(Duration::from_millis(5));
        let points = store.system_timeline(Duration::from_millis(0)).unwrap();
        assert!(points.is_empty());
    }
}
