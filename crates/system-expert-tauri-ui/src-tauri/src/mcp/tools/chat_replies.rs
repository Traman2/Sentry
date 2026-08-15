//! The agent's write-back into a chat space: a reply, progress, or a failure.

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    tool, tool_router, ErrorData as McpError,
};
use tauri::Emitter;

use super::helpers::{blocking, db_error, json_ok};
use crate::mcp::SystemExpertMcp;

/// Emitted when an agent changes something the UI is displaying, so the frontend can refresh
/// rather than wait out its polling interval.
pub const EVENT_CHAT_UPDATED: &str = "mcp://chat-updated";
/// A step the agent took on the way to an answer. Deliberately not written to the database:
/// these describe work in progress, and once the reply lands they are scaffolding. Keeping
/// them purely in flight means the final answer replaces them with no cleanup, and a
/// transcript reopened later shows the answer rather than a log of how it was reached.
pub const EVENT_THINKING: &str = "mcp://thinking";

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PostMessageParams {
    /// The chat space to post into.
    pub chat_space_id: i64,
    /// The assistant message body, as the user will see it in the app.
    pub content: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ThinkingStepParams {
    /// The chat space this work belongs to.
    pub chat_space_id: i64,
    /// One short line, present tense, e.g. "Reading running processes".
    pub step: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PostErrorParams {
    /// The chat space the failed turn belongs to.
    pub chat_space_id: i64,
    /// One line the user can act on, e.g. "Groq rate limit exceeded".
    pub message: String,
    /// The full trace or provider response, revealed behind a disclosure in the UI.
    #[serde(default)]
    pub details: Option<String>,
}

#[tool_router(vis = "pub(crate)", router = tool_router_chat_replies)]
impl SystemExpertMcp {
    #[tool(
        description = "Post an assistant message into a chat space. The user sees it in the \
                       app's chat panel immediately."
    )]
    async fn post_assistant_message(
        &self,
        Parameters(params): Parameters<PostMessageParams>,
    ) -> Result<CallToolResult, McpError> {
        let chat = self.chat.clone();
        let chat_space_id = params.chat_space_id;
        let content = params.content;
        let message = blocking(move || {
            chat.append_message(chat_space_id, "assistant", &content, None)
                .map_err(db_error)
        })
        .await?;

        // Tell the frontend to re-read the space rather than wait for its next poll.
        let _ = self.app.emit(EVENT_CHAT_UPDATED, chat_space_id);
        json_ok(&message)
    }

    #[tool(
        description = "Report progress while working on a reply, so the user sees what you \
                       are doing instead of an unexplained wait. Not saved to the \
                       transcript — the final reply replaces these. Send one short line per \
                       step as you go."
    )]
    async fn post_thinking_step(
        &self,
        Parameters(params): Parameters<ThinkingStepParams>,
    ) -> Result<CallToolResult, McpError> {
        // Straight to the frontend, never to the database.
        let _ = self.app.emit(
            EVENT_THINKING,
            serde_json::json!({
                "chat_space_id": params.chat_space_id,
                "step": params.step,
            }),
        );
        Ok(CallToolResult::success(vec![ContentBlock::text("ok")]))
    }

    #[tool(
        description = "Report that a turn failed, so the user sees the reason instead of a \
                       chat that waits forever. Call this whenever you cannot produce a \
                       reply — `message` is the one-line cause, `details` the full trace."
    )]
    async fn post_error_message(
        &self,
        Parameters(params): Parameters<PostErrorParams>,
    ) -> Result<CallToolResult, McpError> {
        let chat = self.chat.clone();
        let chat_space_id = params.chat_space_id;
        let PostErrorParams {
            message, details, ..
        } = params;
        let posted = blocking(move || {
            chat.append_message(chat_space_id, "error", &message, details.as_deref())
                .map_err(db_error)
        })
        .await?;

        let _ = self.app.emit(EVENT_CHAT_UPDATED, chat_space_id);
        json_ok(&posted)
    }
}
