//! Row types returned by [`super::ChatStore`].

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ChatSpace {
    pub id: i64,
    pub title: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatMessage {
    pub id: i64,
    pub chat_space_id: i64,
    /// `"user"`, `"assistant"`, or `"error"`.
    ///
    /// An `"error"` turn is how a failed agent reply is recorded. It has to be a real
    /// message rather than transient UI state: the agent runs in another process, so if a
    /// turn dies there with nothing written back, the chat panel waits forever on a reply
    /// that is never coming.
    pub role: String,
    pub content: String,
    /// Long-form supporting text — a stack trace for an `"error"` turn. Kept out of
    /// `content` so the UI can show a one-line summary and reveal the rest on demand.
    pub details: Option<String>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatSpaceDetail {
    pub id: i64,
    pub title: String,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub messages: Vec<ChatMessage>,
}
