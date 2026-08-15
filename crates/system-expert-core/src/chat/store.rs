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
        add_details_column(&conn)?;
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

    /// Records a user message and — if this was the space's first message — sets the
    /// space's title to `content`. Returns the space's full, refreshed detail.
    ///
    /// Deliberately does **not** write a reply. The agent answers by calling
    /// [`ChatStore::append_message`] over MCP, so the returned detail ends on the user's
    /// turn and the UI shows a pending state until the real reply lands. An earlier version
    /// fabricated `You asked "..."` here so the panel wasn't silent with no agent attached;
    /// that placeholder now just races the real answer.
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
        insert_message(&tx, chat_space_id, "user", content, None, now)?;

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

    /// Appends a single message with an explicit `role` and bumps the space's
    /// `updated_at_ms` so it sorts to the front of the sidebar.
    ///
    /// This is the write path for the external agent — both its replies (`"assistant"`) and
    /// its failures (`"error"`, where `details` carries the trace). Unlike
    /// [`ChatStore::send_message`] it never touches the space's title, so a space whose
    /// first message arrives this way keeps the default `"New Chat"`.
    ///
    /// Errors with a foreign-key violation if no chat space with `chat_space_id`
    /// exists (`foreign_keys` is `ON`).
    /// Appends `"interrupted"` as the space's next message, but only if its last message is
    /// still a user turn awaiting a reply. Returns `None` without writing anything if not —
    /// most notably if the agent's reply landed first.
    ///
    /// Unlike a caller doing that check via [`ChatStore::get_chat_space`] and then calling
    /// [`ChatStore::append_message`] itself, this holds the store's lock for the whole
    /// check-and-append: the agent's own `append_message` call (posting its reply over MCP,
    /// from another thread) cannot land in the gap between them and get overwritten by a
    /// stale interrupt.
    pub fn interrupt_if_awaiting(
        &self,
        chat_space_id: i64,
        content: &str,
    ) -> rusqlite::Result<Option<ChatSpaceDetail>> {
        let mut conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let tx = conn.transaction()?;

        let awaiting: Option<bool> = tx
            .query_row(
                "SELECT role = 'user' FROM chat_messages
                 WHERE chat_space_id = ?1
                 ORDER BY created_at_ms DESC, id DESC
                 LIMIT 1",
                [chat_space_id],
                |row| row.get(0),
            )
            .optional()?;

        if !awaiting.unwrap_or(false) {
            return Ok(None);
        }

        let now = now_ms();
        insert_message(&tx, chat_space_id, "interrupted", content, None, now)?;
        tx.execute(
            "UPDATE chat_spaces SET updated_at_ms = ?1 WHERE id = ?2",
            rusqlite::params![now, chat_space_id],
        )?;

        let detail = get_chat_space_detail(&tx, chat_space_id)?;
        tx.commit()?;
        Ok(detail)
    }

    pub fn append_message(
        &self,
        chat_space_id: i64,
        role: &str,
        content: &str,
        details: Option<&str>,
    ) -> rusqlite::Result<ChatMessage> {
        let mut conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let tx = conn.transaction()?;

        let now = now_ms();
        insert_message(&tx, chat_space_id, role, content, details, now)?;
        let id = tx.last_insert_rowid();
        tx.execute(
            "UPDATE chat_spaces SET updated_at_ms = ?1 WHERE id = ?2",
            rusqlite::params![now, chat_space_id],
        )?;
        tx.commit()?;

        Ok(ChatMessage {
            id,
            chat_space_id,
            role: role.to_string(),
            content: content.to_string(),
            details: details.map(str::to_string),
            created_at_ms: now,
        })
    }
}

fn insert_message(
    conn: &Connection,
    chat_space_id: i64,
    role: &str,
    content: &str,
    details: Option<&str>,
    now: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO chat_messages (chat_space_id, role, content, details, created_at_ms)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![chat_space_id, role, content, details, now],
    )?;
    Ok(())
}

/// Adds `chat_messages.details` to databases created before it existed.
///
/// `CREATE TABLE IF NOT EXISTS` above is a no-op on an existing database, so a new column has
/// to be added separately or every install predating it would break on the first query.
/// SQLite has no `ADD COLUMN IF NOT EXISTS`, hence the `table_info` check.
fn add_details_column(conn: &Connection) -> rusqlite::Result<()> {
    let mut stmt = conn.prepare("SELECT name FROM pragma_table_info('chat_messages')")?;
    let exists = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .any(|name| matches!(name, Ok(name) if name == "details"));
    drop(stmt);

    if !exists {
        conn.execute("ALTER TABLE chat_messages ADD COLUMN details TEXT", [])?;
    }
    Ok(())
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
        "SELECT id, chat_space_id, role, content, details, created_at_ms
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
                details: row.get(4)?,
                created_at_ms: row.get(5)?,
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
