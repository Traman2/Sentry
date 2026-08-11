//! Timeline queries over recorded history: "give me samples from the last N seconds",
//! either system-wide or for one specific process.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use super::HistoryStore;

#[derive(Debug, Clone, Serialize)]
pub struct SystemSamplePoint {
    pub timestamp_ms: i64,
    pub cpu_usage_percent: f32,
    pub used_memory_bytes: u64,
    pub total_memory_bytes: u64,
    pub used_swap_bytes: u64,
    pub network_rx_bytes_per_sec: u64,
    pub network_tx_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSamplePoint {
    pub timestamp_ms: i64,
    pub pid: u32,
    pub name: String,
    pub cpu_usage_percent: f32,
    pub memory_bytes: u64,
    pub disk_read_bytes_per_sec: u64,
    pub disk_written_bytes_per_sec: u64,
}

impl HistoryStore {
    /// System-wide aggregate samples from the last `since` (e.g.
    /// `Duration::from_secs(3600)` for "the last hour"), oldest first.
    pub fn system_timeline(&self, since: Duration) -> rusqlite::Result<Vec<SystemSamplePoint>> {
        let cutoff_ms = cutoff_ms(since);
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn.prepare(
            "SELECT timestamp_ms, cpu_usage_percent, used_memory_bytes, total_memory_bytes,
                    used_swap_bytes, network_rx_bytes_per_sec, network_tx_bytes_per_sec
             FROM system_samples
             WHERE timestamp_ms >= ?1
             ORDER BY timestamp_ms ASC",
        )?;
        let rows = stmt.query_map([cutoff_ms], |row| {
            Ok(SystemSamplePoint {
                timestamp_ms: row.get(0)?,
                cpu_usage_percent: row.get(1)?,
                used_memory_bytes: row.get::<_, i64>(2)? as u64,
                total_memory_bytes: row.get::<_, i64>(3)? as u64,
                used_swap_bytes: row.get::<_, i64>(4)? as u64,
                network_rx_bytes_per_sec: row.get::<_, i64>(5)? as u64,
                network_tx_bytes_per_sec: row.get::<_, i64>(6)? as u64,
            })
        })?;
        rows.collect()
    }

    /// Samples for a single `pid` from the last `since`, oldest first. Empty if the
    /// pid was never recorded or has aged out of the retention window.
    pub fn process_timeline(
        &self,
        pid: u32,
        since: Duration,
    ) -> rusqlite::Result<Vec<ProcessSamplePoint>> {
        let cutoff_ms = cutoff_ms(since);
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn.prepare(
            "SELECT timestamp_ms, pid, name, cpu_usage_percent, memory_bytes,
                    disk_read_bytes_per_sec, disk_written_bytes_per_sec
             FROM process_samples
             WHERE pid = ?1 AND timestamp_ms >= ?2
             ORDER BY timestamp_ms ASC",
        )?;
        let rows = stmt.query_map(rusqlite::params![pid, cutoff_ms], |row| {
            Ok(ProcessSamplePoint {
                timestamp_ms: row.get(0)?,
                pid: row.get::<_, i64>(1)? as u32,
                name: row.get(2)?,
                cpu_usage_percent: row.get(3)?,
                memory_bytes: row.get::<_, i64>(4)? as u64,
                disk_read_bytes_per_sec: row.get::<_, i64>(5)? as u64,
                disk_written_bytes_per_sec: row.get::<_, i64>(6)? as u64,
            })
        })?;
        rows.collect()
    }

    /// Samples for a fixed set of `pids` from the last `since`, oldest first —
    /// backs "track this app" charts, where an app is a specific set of pids
    /// captured at the moment tracking started (not a live re-grouping by name, so
    /// a same-named process that spawns later is not silently folded in).
    /// Empty if `pids` is empty.
    pub fn process_timeline_for_pids(
        &self,
        pids: &[u32],
        since: Duration,
    ) -> rusqlite::Result<Vec<ProcessSamplePoint>> {
        if pids.is_empty() {
            return Ok(Vec::new());
        }

        let cutoff_ms = cutoff_ms(since);
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let placeholders = vec!["?"; pids.len()].join(",");
        let sql = format!(
            "SELECT timestamp_ms, pid, name, cpu_usage_percent, memory_bytes,
                    disk_read_bytes_per_sec, disk_written_bytes_per_sec
             FROM process_samples
             WHERE pid IN ({placeholders}) AND timestamp_ms >= ?
             ORDER BY timestamp_ms ASC"
        );

        let mut stmt = conn.prepare(&sql)?;
        let params: Vec<i64> = pids
            .iter()
            .map(|&pid| pid as i64)
            .chain(std::iter::once(cutoff_ms))
            .collect();
        let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| {
            Ok(ProcessSamplePoint {
                timestamp_ms: row.get(0)?,
                pid: row.get::<_, i64>(1)? as u32,
                name: row.get(2)?,
                cpu_usage_percent: row.get(3)?,
                memory_bytes: row.get::<_, i64>(4)? as u64,
                disk_read_bytes_per_sec: row.get::<_, i64>(5)? as u64,
                disk_written_bytes_per_sec: row.get::<_, i64>(6)? as u64,
            })
        })?;
        rows.collect()
    }
}

fn cutoff_ms(since: Duration) -> i64 {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    now_ms.saturating_sub(since.as_millis() as i64)
}
