//! The one tool that changes the machine rather than just reading it.

use rmcp::{
    handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
    ErrorData as McpError,
};
use tauri::Emitter;

use super::helpers::{blocking, json_ok, tool_error};
use crate::mcp::types::kill_guard;
use crate::mcp::SystemExpertMcp;

pub const EVENT_PROCESSES_CHANGED: &str = "mcp://processes-changed";

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct KillProcessParams {
    /// The process id to terminate.
    pub pid: u32,
    /// The process name you believe this pid belongs to. Strongly recommended: pids are
    /// recycled by the OS, and passing this makes the call refuse rather than terminate a
    /// different process that inherited the number.
    #[serde(default)]
    pub expect_name: Option<String>,
    /// Override the refusal of critical OS processes and of System-Expert itself. Almost never
    /// correct.
    #[serde(default)]
    pub force: bool,
}

#[tool_router(vis = "pub(crate)", router = tool_router_action)]
impl SystemExpertMcp {
    #[tool(
        description = "Terminate a process. This is irreversible and affects the user's real \
                       machine — look the process up first and pass expect_name from what you \
                       saw, so a recycled pid cannot make this kill the wrong process. \
                       Critical OS processes and System-Expert itself are refused unless force is set."
    )]
    async fn kill_process(
        &self,
        Parameters(params): Parameters<KillProcessParams>,
    ) -> Result<CallToolResult, McpError> {
        if let Some(reason) = kill_guard(params.pid, params.expect_name.as_deref(), params.force) {
            return Ok(tool_error(format!("refused: {reason}")));
        }

        let monitor = self.monitor.clone();
        let pid = params.pid;
        let expect_name = params.expect_name.clone();
        let outcome = blocking(move || {
            let mut monitor = monitor.lock().unwrap_or_else(|e| e.into_inner());
            Ok(monitor.kill_process(pid, expect_name.as_deref(), None))
        })
        .await?;

        if outcome.delivered {
            let _ = self.app.emit(EVENT_PROCESSES_CHANGED, pid);
            json_ok(&outcome)
        } else {
            // The process survived (or never existed). That's the caller's problem to read,
            // not a protocol failure — return the full outcome so they can see why.
            let text = serde_json::to_string_pretty(&outcome).unwrap_or(outcome.message);
            Ok(tool_error(text))
        }
    }
}
