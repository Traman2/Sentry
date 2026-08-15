//! The one read-only tool exposing the app's own agent settings to the agent itself.

use rmcp::{model::CallToolResult, tool, tool_router, ErrorData as McpError};
use tauri::Manager;

use super::helpers::json_ok;
use crate::mcp::SentryMcp;

#[tool_router(vis = "pub(crate)", router = tool_router_agent_config)]
impl SentryMcp {
    #[tool(
        description = "The agent settings the user has selected in the desktop app, and the \
                       WebSocket that pushes changes. Connect to `events_url` and react to \
                       what arrives rather than calling this repeatedly — the app announces \
                       new chat messages and model changes there."
    )]
    async fn get_agent_config(&self) -> Result<CallToolResult, McpError> {
        // Read through the AppHandle rather than holding a copy: the picker writes to the
        // same Tauri state the `set_agent_model` command writes, so there is exactly one
        // source of truth for which model is selected.
        let model = self
            .app
            .try_state::<crate::agent::AgentState>()
            .map(|state| state.lock().model().to_string())
            .unwrap_or_else(|| "qwen".to_string());
        json_ok(&serde_json::json!({
            "model": model,
            "events_url": crate::mcp::server::events_url(),
        }))
    }
}
