//! Reading and creating chat spaces. See `chat_replies` for the agent's write-back.

use rmcp::{
    handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
    ErrorData as McpError,
};
use tauri::Emitter;

use super::chat_replies::EVENT_CHAT_UPDATED;
use super::helpers::{blocking, db_error, json_ok, tool_error};
use crate::mcp::SystemExpertMcp;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ChatSpaceIdParams {
    /// The chat space id.
    pub id: i64,
}

#[tool_router(vis = "pub(crate)", router = tool_router_chat)]
impl SystemExpertMcp {
    #[tool(description = "List the app's chat spaces, most recently active first.")]
    async fn list_chat_spaces(&self) -> Result<CallToolResult, McpError> {
        let chat = self.chat.clone();
        let spaces = blocking(move || chat.list_chat_spaces().map_err(db_error)).await?;
        json_ok(&spaces)
    }

    #[tool(description = "Read one chat space with its full message history, oldest first.")]
    async fn get_chat_space(
        &self,
        Parameters(params): Parameters<ChatSpaceIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let chat = self.chat.clone();
        let id = params.id;
        let detail = blocking(move || chat.get_chat_space(id).map_err(db_error)).await?;

        match detail {
            Some(detail) => json_ok(&detail),
            None => Ok(tool_error(format!("no chat space with id {id}"))),
        }
    }

    #[tool(description = "Create a new, empty chat space in the app.")]
    async fn create_chat_space(&self) -> Result<CallToolResult, McpError> {
        let chat = self.chat.clone();
        let space = blocking(move || chat.create_chat_space().map_err(db_error)).await?;
        let _ = self.app.emit(EVENT_CHAT_UPDATED, space.id);
        json_ok(&space)
    }
}
