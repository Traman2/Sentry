//! Row types returned by [`super::McpUsageStore`].

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct McpClient {
    pub id: i64,
    /// Dedup identity — see [`super::identify::identify`]. Not shown in the UI directly;
    /// `display_name` is.
    pub key: String,
    pub display_name: String,
    /// Best-effort classification for a badge/icon: `"claude-code"`, `"claude-desktop"`,
    /// `"python-agent"`, `"chatgpt"`, `"grok"`, or `"unknown"`. Never blocks a client from
    /// being recorded — an unrecognised one just gets `"unknown"` and keeps its raw
    /// `display_name`.
    pub kind: String,
    /// The raw `clientInfo.name` this client sent at `initialize`, if any.
    pub protocol_name: Option<String>,
    pub protocol_version: Option<String>,
    pub last_pid: Option<u32>,
    pub last_process_name: Option<String>,
    pub call_count: i64,
    pub first_seen_ms: i64,
    pub last_seen_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpToolCall {
    pub id: i64,
    pub client_id: i64,
    pub tool_name: String,
    /// The call's arguments, serialized as JSON and length-capped — see
    /// [`super::store::MAX_PARAMS_JSON_LEN`]. `None` if the call took no arguments or
    /// serialization failed.
    pub params_json: Option<String>,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    /// `"ok"` (the tool ran, whether or not its own result was a tool-level error the caller
    /// sees), or `"protocol_error"` (the call never reached a tool — e.g. unknown tool name).
    pub status: String,
    pub duration_ms: i64,
    /// Set only for `"protocol_error"` — the message accompanying that failure.
    pub error_message: Option<String>,
    pub created_at_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct McpClientDetail {
    pub id: i64,
    pub key: String,
    pub display_name: String,
    pub kind: String,
    pub protocol_name: Option<String>,
    pub protocol_version: Option<String>,
    pub last_pid: Option<u32>,
    pub last_process_name: Option<String>,
    pub call_count: i64,
    pub first_seen_ms: i64,
    pub last_seen_ms: i64,
    /// Most recent first.
    pub calls: Vec<McpToolCall>,
}
