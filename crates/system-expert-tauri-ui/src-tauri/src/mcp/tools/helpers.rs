//! Small pieces every tool handler needs, shared across `tools/*`.
//!
//! **Everything blocking goes through [`blocking`].** `system-expert-core` is entirely synchronous —
//! `Monitor::snapshot` refreshes the whole process table, and every store method holds a
//! `std::sync::Mutex` across SQLite I/O. Calling those directly from an async tool body would
//! park an axum worker thread for the duration.

use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::ErrorData as McpError;
use serde::Serialize;

/// Runs a blocking `system-expert-core` call on the blocking pool.
pub(super) async fn blocking<T, F>(f: F) -> Result<T, McpError>
where
    F: FnOnce() -> Result<T, McpError> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| McpError::internal_error(format!("worker task failed: {e}"), None))?
}

/// Serializes `value` as the tool's text result.
pub(super) fn json_ok<T: Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("failed to serialize result: {e}"), None))?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

/// A tool-level failure the caller should read, as opposed to `Err(McpError)`, which MCP
/// clients render opaquely.
pub(super) fn tool_error(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.into())])
}

/// Generic over the error type so this crate doesn't need a direct `rusqlite` dependency
/// just to name `system-expert-core`'s error.
pub(super) fn db_error<E: std::fmt::Display>(e: E) -> McpError {
    McpError::internal_error(format!("database error: {e}"), None)
}

/// Shared by `history` and `tracking`, whose `max_points` params both downsample to the same
/// default cap.
pub(super) fn default_max_points() -> usize {
    120
}
