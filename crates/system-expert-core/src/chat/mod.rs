//! Persistent chat spaces and their message turns, backed by SQLite (mirrors the
//! [`super::history`] module's storage approach).
//!
//! A chat space is a single conversation thread: an id, a title (initially set to
//! the user's first message, later expected to be replaced by an AI-generated
//! summary once the chatbot agent exists), and its ordered messages.

mod models;
mod store;

pub use models::{ChatMessage, ChatSpace, ChatSpaceDetail};
pub use store::ChatStore;

#[cfg(test)]
mod tests;
