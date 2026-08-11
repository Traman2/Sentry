//! [`TrackingStore`]: bookkeeping for "Track" sessions — which pids are being
//! tracked, since when, and (once ended) where their archived samples live on
//! disk. The actual CPU/memory samples themselves live in
//! [`crate::history::HistoryStore`]; this store only tracks the session metadata.

use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, Row};

use super::models::TrackedProcess;

/// A SQLite-backed store of tracked-process sessions.
///
/// Safe to share across threads via `Arc<TrackingStore>` (or bare, as
/// Tauri-managed state): all access goes through an internal `Mutex<Connection>`.
pub struct TrackingStore {
    conn: Mutex<Connection>,
}

impl TrackingStore {
    /// Opens (creating if needed) a tracking database at `path` and ensures its
    /// schema exists.
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS tracked_processes (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                name             TEXT NOT NULL,
                pids               TEXT NOT NULL,
                started_at_ms        INTEGER NOT NULL,
                ended_at_ms             INTEGER,
                status                    TEXT NOT NULL,
                archive_path                TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_tracked_processes_name
                ON tracked_processes(name);
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

    /// Starts a new tracking session over `pids`, which are captured as a fixed
    /// set for this session's lifetime (an app being tracked is this exact set of
    /// pids, not a live re-grouping by name).
    pub fn start(&self, name: &str, pids: &[i64]) -> rusqlite::Result<TrackedProcess> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let now = now_ms();
        let pids_json = serde_json::to_string(pids).unwrap_or_else(|_| "[]".to_string());
        conn.execute(
            "INSERT INTO tracked_processes (name, pids, started_at_ms, status)
             VALUES (?1, ?2, ?3, 'active')",
            rusqlite::params![name, pids_json, now],
        )?;
        Ok(TrackedProcess {
            id: conn.last_insert_rowid(),
            name: name.to_string(),
            pids: pids.to_vec(),
            started_at_ms: now,
            ended_at_ms: None,
            status: "active".to_string(),
            archive_path: None,
        })
    }

    /// Lists tracked sessions, most recently started first. Filters to sessions for
    /// `name` when given — backs the Details View's "tracking history" for one
    /// process/app.
    pub fn list(&self, name: Option<&str>) -> rusqlite::Result<Vec<TrackedProcess>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = match name {
            Some(_) => conn.prepare(
                "SELECT id, name, pids, started_at_ms, ended_at_ms, status, archive_path
                 FROM tracked_processes
                 WHERE name = ?1
                 ORDER BY started_at_ms DESC",
            )?,
            None => conn.prepare(
                "SELECT id, name, pids, started_at_ms, ended_at_ms, status, archive_path
                 FROM tracked_processes
                 ORDER BY started_at_ms DESC",
            )?,
        };
        let rows = match name {
            Some(name) => stmt.query_map(rusqlite::params![name], row_to_tracked)?,
            None => stmt.query_map([], row_to_tracked)?,
        };
        rows.collect()
    }

    /// Fetches one tracked session. `None` if no session with `id` exists.
    pub fn get(&self, id: i64) -> rusqlite::Result<Option<TrackedProcess>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.query_row(
            "SELECT id, name, pids, started_at_ms, ended_at_ms, status, archive_path
             FROM tracked_processes WHERE id = ?1",
            [id],
            row_to_tracked,
        )
        .optional()
    }

    /// Marks a session ended (manually stopped, or because every tracked pid
    /// disappeared). A no-op if it was already ended. Returns the updated session,
    /// or `None` if no session with `id` exists.
    pub fn end(&self, id: i64) -> rusqlite::Result<Option<TrackedProcess>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "UPDATE tracked_processes SET status = 'ended', ended_at_ms = ?1
             WHERE id = ?2 AND status = 'active'",
            rusqlite::params![now_ms(), id],
        )?;
        conn.query_row(
            "SELECT id, name, pids, started_at_ms, ended_at_ms, status, archive_path
             FROM tracked_processes WHERE id = ?1",
            [id],
            row_to_tracked,
        )
        .optional()
    }

    /// Records where an ended session's archived samples were written.
    pub fn set_archive_path(&self, id: i64, path: &str) -> rusqlite::Result<Option<TrackedProcess>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "UPDATE tracked_processes SET archive_path = ?1 WHERE id = ?2",
            rusqlite::params![path, id],
        )?;
        conn.query_row(
            "SELECT id, name, pids, started_at_ms, ended_at_ms, status, archive_path
             FROM tracked_processes WHERE id = ?1",
            [id],
            row_to_tracked,
        )
        .optional()
    }

    /// Deletes a tracked session and returns it (so the caller can also remove its
    /// archive file, if any) — `None` if no session with `id` existed.
    pub fn delete(&self, id: i64) -> rusqlite::Result<Option<TrackedProcess>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let existing = conn
            .query_row(
                "SELECT id, name, pids, started_at_ms, ended_at_ms, status, archive_path
                 FROM tracked_processes WHERE id = ?1",
                [id],
                row_to_tracked,
            )
            .optional()?;
        if existing.is_some() {
            conn.execute("DELETE FROM tracked_processes WHERE id = ?1", [id])?;
        }
        Ok(existing)
    }
}

fn row_to_tracked(row: &Row) -> rusqlite::Result<TrackedProcess> {
    let pids_json: String = row.get(2)?;
    let pids: Vec<i64> = serde_json::from_str(&pids_json).unwrap_or_default();
    Ok(TrackedProcess {
        id: row.get(0)?,
        name: row.get(1)?,
        pids,
        started_at_ms: row.get(3)?,
        ended_at_ms: row.get(4)?,
        status: row.get(5)?,
        archive_path: row.get(6)?,
    })
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
