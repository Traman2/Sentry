//! Which MCP clients have called this app's tools, and what they called — backed by SQLite
//! (mirrors [`super::chat`]'s storage approach).
//!
//! A client is one distinct caller identity — Claude Code, Claude Desktop, the bundled Python
//! agent, or generically anything else an MCP client sends `clientInfo` for (or, failing
//! that, whatever OS process was on the other end of the loopback socket). Each client has an
//! id, a display name, and its ordered tool-call history. See [`identify::identify`] for how
//! a client's identity is derived.

mod identify;
mod models;
mod store;

pub use identify::ClientIdentity;
pub use models::{McpClient, McpClientDetail, McpToolCall};
pub use store::{CallStatus, McpUsageStore};

#[cfg(test)]
mod tests;
