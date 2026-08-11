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
    pub role: String,
    pub content: String,
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
