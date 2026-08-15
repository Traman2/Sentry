//! [`McpUsageStore`]: the SQLite-backed store of MCP clients and the tool calls they've made.

use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, Row, params};

use super::identify::{ClientIdentity, identify};
use super::models::{McpClient, McpClientDetail, McpToolCall};

/// Tool-call arguments are logged for debugging visibility, not as a durable record of
/// exactly what was sent — capped well short of anything that would make this store a second
/// copy of large payloads (e.g. a chat message forwarded through `post_assistant_message`).
pub(crate) const MAX_PARAMS_JSON_LEN: usize = 2000;

/// How many of a client's most recent tool calls [`McpUsageStore::get_client_usage`] returns.
const MAX_CALLS_PER_CLIENT: usize = 500;

/// Whether a tool call reached and ran a tool, or never got that far.
///
/// This is about the MCP *protocol* layer, not the tool's own success/failure — a tool that
/// ran and returned `CallToolResult::error(...)` (a refused kill, a missing id) is still
/// [`CallStatus::Ok`] here: the call was served. [`CallStatus::ProtocolError`] is for requests
/// that never reached a tool body at all (unknown tool name, malformed request).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallStatus {
    Ok,
    ProtocolError,
}

impl CallStatus {
    fn as_str(self) -> &'static str {
        match self {
            CallStatus::Ok => "ok",
            CallStatus::ProtocolError => "protocol_error",
        }
    }
}

/// A SQLite-backed store of MCP clients and their tool-call history.
///
/// Safe to share across threads via `Arc<McpUsageStore>` (or bare, as Tauri-managed state):
/// all access goes through an internal `Mutex<Connection>`.
pub struct McpUsageStore {
    conn: Mutex<Connection>,
}

impl McpUsageStore {
    /// Opens (creating if needed) an MCP-usage database at `path` and ensures its schema
    /// exists.
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS mcp_clients (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                key                  TEXT NOT NULL UNIQUE,
                display_name           TEXT NOT NULL,
                kind                      TEXT NOT NULL,
                protocol_name              TEXT,
                protocol_version              TEXT,
                last_pid                        INTEGER,
                last_process_name                 TEXT,
                call_count                          INTEGER NOT NULL DEFAULT 0,
                first_seen_ms                         INTEGER NOT NULL,
                last_seen_ms                            INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS mcp_tool_calls (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                client_id            INTEGER NOT NULL
                    REFERENCES mcp_clients(id) ON DELETE CASCADE,
                tool_name               TEXT NOT NULL,
                params_json               TEXT,
                pid                         INTEGER,
                process_name                  TEXT,
                status                          TEXT NOT NULL,
                duration_ms                       INTEGER NOT NULL,
                error_message                       TEXT,
                created_at_ms                         INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_mcp_tool_calls_client
                ON mcp_tool_calls(client_id, created_at_ms);
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

    /// Records one tool call: upserts the calling client (by [`identify`]'s dedup key,
    /// bumping its `last_seen_ms`/`last_pid`/`last_process_name`/`call_count`) and inserts
    /// the call row, transactionally.
    pub fn record_call(
        &self,
        identity: &ClientIdentity,
        tool_name: &str,
        params_json: Option<&str>,
        status: CallStatus,
        duration_ms: i64,
        error_message: Option<&str>,
    ) -> rusqlite::Result<(McpClient, McpToolCall)> {
        let (key, display_name, kind) = identify(identity);
        let params_json = params_json.map(|s| truncate(s, MAX_PARAMS_JSON_LEN));

        let mut conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let tx = conn.transaction()?;
        let now = now_ms();

        tx.execute(
            "INSERT INTO mcp_clients
                (key, display_name, kind, protocol_name, protocol_version, last_pid,
                 last_process_name, call_count, first_seen_ms, last_seen_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8, ?8)
             ON CONFLICT(key) DO UPDATE SET
                display_name = excluded.display_name,
                kind = excluded.kind,
                protocol_name = excluded.protocol_name,
                protocol_version = excluded.protocol_version,
                last_pid = excluded.last_pid,
                last_process_name = excluded.last_process_name,
                call_count = call_count + 1,
                last_seen_ms = excluded.last_seen_ms",
            params![
                key,
                display_name,
                kind,
                identity.protocol_name,
                identity.protocol_version,
                identity.pid,
                identity.process_name,
                now,
            ],
        )?;

        let client = tx.query_row(
            &format!("{CLIENT_COLUMNS} FROM mcp_clients WHERE key = ?1"),
            [&key],
            client_from_row,
        )?;

        tx.execute(
            "INSERT INTO mcp_tool_calls
                (client_id, tool_name, params_json, pid, process_name, status, duration_ms,
                 error_message, created_at_ms)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                client.id,
                tool_name,
                params_json,
                identity.pid,
                identity.process_name,
                status.as_str(),
                duration_ms,
                error_message,
                now,
            ],
        )?;
        let call_id = tx.last_insert_rowid();
        tx.commit()?;

        let call = McpToolCall {
            id: call_id,
            client_id: client.id,
            tool_name: tool_name.to_string(),
            params_json,
            pid: identity.pid,
            process_name: identity.process_name.clone(),
            status: status.as_str().to_string(),
            duration_ms,
            error_message: error_message.map(str::to_string),
            created_at_ms: now,
        };

        Ok((client, call))
    }

    /// Deletes a client and its tool-call history (via `ON DELETE CASCADE`). Returns whether
    /// a client with `id` existed. Lets a user clear out a stale or misidentified entry —
    /// e.g. from a client that only ever connected once while its identification logic was
    /// still landing — without it lingering in the panel forever.
    pub fn delete_client(&self, id: i64) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let deleted = conn.execute("DELETE FROM mcp_clients WHERE id = ?1", [id])?;
        Ok(deleted > 0)
    }

    /// Lists all known clients, most recently active first.
    pub fn list_clients(&self) -> rusqlite::Result<Vec<McpClient>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn.prepare(&format!(
            "{CLIENT_COLUMNS} FROM mcp_clients ORDER BY last_seen_ms DESC"
        ))?;
        let rows = stmt.query_map([], client_from_row)?;
        rows.collect()
    }

    /// Fetches one client with its most recent tool calls (newest first, capped at
    /// [`MAX_CALLS_PER_CLIENT`]). `None` if no client with `id` exists.
    pub fn get_client_usage(&self, id: i64) -> rusqlite::Result<Option<McpClientDetail>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        let client = conn
            .query_row(
                &format!("{CLIENT_COLUMNS} FROM mcp_clients WHERE id = ?1"),
                [id],
                client_from_row,
            )
            .optional()?;

        let Some(client) = client else {
            return Ok(None);
        };

        let mut stmt = conn.prepare(
            "SELECT id, client_id, tool_name, params_json, pid, process_name, status,
                    duration_ms, error_message, created_at_ms
             FROM mcp_tool_calls
             WHERE client_id = ?1
             ORDER BY created_at_ms DESC, id DESC
             LIMIT ?2",
        )?;
        let calls = stmt
            .query_map(params![id, MAX_CALLS_PER_CLIENT as i64], |row| {
                Ok(McpToolCall {
                    id: row.get(0)?,
                    client_id: row.get(1)?,
                    tool_name: row.get(2)?,
                    params_json: row.get(3)?,
                    pid: row.get(4)?,
                    process_name: row.get(5)?,
                    status: row.get(6)?,
                    duration_ms: row.get(7)?,
                    error_message: row.get(8)?,
                    created_at_ms: row.get(9)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(Some(McpClientDetail {
            id: client.id,
            key: client.key,
            display_name: client.display_name,
            kind: client.kind,
            protocol_name: client.protocol_name,
            protocol_version: client.protocol_version,
            last_pid: client.last_pid,
            last_process_name: client.last_process_name,
            call_count: client.call_count,
            first_seen_ms: client.first_seen_ms,
            last_seen_ms: client.last_seen_ms,
            calls,
        }))
    }
}

const CLIENT_COLUMNS: &str = "SELECT id, key, display_name, kind, protocol_name, protocol_version, \
     last_pid, last_process_name, call_count, first_seen_ms, last_seen_ms";

fn client_from_row(row: &Row) -> rusqlite::Result<McpClient> {
    Ok(McpClient {
        id: row.get(0)?,
        key: row.get(1)?,
        display_name: row.get(2)?,
        kind: row.get(3)?,
        protocol_name: row.get(4)?,
        protocol_version: row.get(5)?,
        last_pid: row.get(6)?,
        last_process_name: row.get(7)?,
        call_count: row.get(8)?,
        first_seen_ms: row.get(9)?,
        last_seen_ms: row.get(10)?,
    })
}

/// Truncates `s` to at most `max_len` bytes on a `char` boundary, so a capped payload never
/// splits a multi-byte UTF-8 character into invalid bytes.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        return s.to_string();
    }
    let mut end = max_len;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
