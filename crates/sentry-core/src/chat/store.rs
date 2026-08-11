//! [`ChatStore`]: the SQLite-backed store of chat spaces and their messages.

use std::path::Path;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension};

use super::models::{ChatMessage, ChatSpace, ChatSpaceDetail};

const DEFAULT_TITLE: &str = "New Chat";

/// A SQLite-backed store of chat spaces and their messages.
///
/// Safe to share across threads via `Arc<ChatStore>` (or bare, as Tauri-managed
/// state): all access goes through an internal `Mutex<Connection>`.
pub struct ChatStore {
    conn: Mutex<Connection>,
}

impl ChatStore {
    /// Opens (creating if needed) a chat database at `path` and ensures its schema
    /// exists.
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS chat_spaces (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                title            TEXT NOT NULL,
                created_at_ms      INTEGER NOT NULL,
                updated_at_ms        INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS chat_messages (
                id                 INTEGER PRIMARY KEY AUTOINCREMENT,
                chat_space_id        INTEGER NOT NULL
                    REFERENCES chat_spaces(id) ON DELETE CASCADE,
                role                    TEXT NOT NULL,
                content                   TEXT NOT NULL,
                created_at_ms               INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_chat_messages_space
                ON chat_messages(chat_space_id, created_at_ms);
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

    /// Creates a new, empty chat space titled `"New Chat"`. The title is replaced by
    /// the user's first message the first time [`ChatStore::send_message`] is called
    /// on it.
    pub fn create_chat_space(&self) -> rusqlite::Result<ChatSpace> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let now = now_ms();
        conn.execute(
            "INSERT INTO chat_spaces (title, created_at_ms, updated_at_ms) VALUES (?1, ?2, ?2)",
            rusqlite::params![DEFAULT_TITLE, now],
        )?;
        Ok(ChatSpace {
            id: conn.last_insert_rowid(),
            title: DEFAULT_TITLE.to_string(),
            created_at_ms: now,
            updated_at_ms: now,
        })
    }

    /// Lists all chat spaces, most recently active first.
    pub fn list_chat_spaces(&self) -> rusqlite::Result<Vec<ChatSpace>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn.prepare(
            "SELECT id, title, created_at_ms, updated_at_ms
             FROM chat_spaces
             ORDER BY updated_at_ms DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ChatSpace {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at_ms: row.get(2)?,
                updated_at_ms: row.get(3)?,
            })
        })?;
        rows.collect()
    }

    /// Fetches one chat space with all of its messages, oldest first. `None` if no
    /// chat space with `id` exists.
    pub fn get_chat_space(&self, id: i64) -> rusqlite::Result<Option<ChatSpaceDetail>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        get_chat_space_detail(&conn, id)
    }

    /// Deletes a chat space and all of its messages (via `ON DELETE CASCADE`).
    /// Returns whether a chat space with `id` existed.
    pub fn delete_chat_space(&self, id: i64) -> rusqlite::Result<bool> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let deleted = conn.execute("DELETE FROM chat_spaces WHERE id = ?1", [id])?;
        Ok(deleted > 0)
    }

    /// Records a user message, appends a placeholder assistant reply (a stand-in
    /// until the real chatbot agent is wired up), and — if this was the space's
    /// first message — sets the space's title to `content`. Returns the space's full,
    /// refreshed detail.
    pub fn send_message(
        &self,
        chat_space_id: i64,
        content: &str,
    ) -> rusqlite::Result<ChatSpaceDetail> {
        let mut conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let tx = conn.transaction()?;

        let is_first_message: bool = tx.query_row(
            "SELECT NOT EXISTS(SELECT 1 FROM chat_messages WHERE chat_space_id = ?1)",
            [chat_space_id],
            |row| row.get(0),
        )?;

        let now = now_ms();
        tx.execute(
            "INSERT INTO chat_messages (chat_space_id, role, content, created_at_ms)
             VALUES (?1, 'user', ?2, ?3)",
            rusqlite::params![chat_space_id, content, now],
        )?;

        // Placeholder until the real chatbot agent exists.
        let reply = format!("You asked \"{content}\"");
        tx.execute(
            "INSERT INTO chat_messages (chat_space_id, role, content, created_at_ms)
             VALUES (?1, 'assistant', ?2, ?3)",
            rusqlite::params![chat_space_id, reply, now],
        )?;

        if is_first_message {
            tx.execute(
                "UPDATE chat_spaces SET title = ?1, updated_at_ms = ?2 WHERE id = ?3",
                rusqlite::params![content, now, chat_space_id],
            )?;
        } else {
            tx.execute(
                "UPDATE chat_spaces SET updated_at_ms = ?1 WHERE id = ?2",
                rusqlite::params![now, chat_space_id],
            )?;
        }

        let detail = get_chat_space_detail(&tx, chat_space_id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        tx.commit()?;
        Ok(detail)
    }
}

fn get_chat_space_detail(conn: &Connection, id: i64) -> rusqlite::Result<Option<ChatSpaceDetail>> {
    let space = conn
        .query_row(
            "SELECT id, title, created_at_ms, updated_at_ms FROM chat_spaces WHERE id = ?1",
            [id],
            |row| {
                Ok(ChatSpace {
                    id: row.get(0)?,
                    title: row.get(1)?,
                    created_at_ms: row.get(2)?,
                    updated_at_ms: row.get(3)?,
                })
            },
        )
        .optional()?;

    let Some(space) = space else {
        return Ok(None);
    };

    let mut stmt = conn.prepare(
        "SELECT id, chat_space_id, role, content, created_at_ms
         FROM chat_messages
         WHERE chat_space_id = ?1
         ORDER BY created_at_ms ASC, id ASC",
    )?;
    let messages = stmt
        .query_map([id], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                chat_space_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at_ms: row.get(4)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    Ok(Some(ChatSpaceDetail {
        id: space.id,
        title: space.title,
        created_at_ms: space.created_at_ms,
        updated_at_ms: space.updated_at_ms,
        messages,
    }))
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}
