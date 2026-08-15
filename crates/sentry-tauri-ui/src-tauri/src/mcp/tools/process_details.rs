//! Inspecting one process: its working directory, root, and environment.

use rmcp::{
    handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
    ErrorData as McpError,
};
use serde::Serialize;

use super::helpers::{blocking, json_ok, tool_error};
use crate::mcp::SentryMcp;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProcessDetailsParams {
    /// The process id to inspect.
    pub pid: u32,
    /// Return full `KEY=VALUE` environment strings instead of just variable names. Off by
    /// default because a process's environment routinely holds API tokens and credentials.
    #[serde(default)]
    pub include_environment: bool,
}

#[derive(Serialize)]
struct ProcessDetailsResponse {
    pid: u32,
    name: String,
    current_working_directory: Option<String>,
    root_directory: Option<String>,
    /// Present only when `include_environment` was set.
    environment: Option<Vec<String>>,
    /// Variable names only — always present, and safe to show.
    environment_variable_names: Vec<String>,
}

#[tool_router(vis = "pub(crate)", router = tool_router_process_details)]
impl SentryMcp {
    #[tool(
        description = "Working directory, root directory and environment for one process. \
                       Environment values are withheld unless include_environment is set, \
                       because they commonly contain credentials."
    )]
    async fn get_process_details(
        &self,
        Parameters(params): Parameters<ProcessDetailsParams>,
    ) -> Result<CallToolResult, McpError> {
        let monitor = self.monitor.clone();
        let pid = params.pid;
        let details = blocking(move || {
            let mut monitor = monitor.lock().unwrap_or_else(|e| e.into_inner());
            // `process_details` refreshes just this pid — deliberately not a full snapshot,
            // which would both cost a whole-table refresh and reset this Monitor's CPU delta
            // baseline as a side effect of answering a question about one process.
            Ok(monitor.process_details(pid))
        })
        .await?;

        let Some(details) = details else {
            return Ok(tool_error(format!("no process with pid {pid}")));
        };

        let names: Vec<String> = details
            .environment
            .iter()
            .map(|entry| entry.split('=').next().unwrap_or(entry).to_string())
            .collect();

        json_ok(&ProcessDetailsResponse {
            pid: details.pid,
            name: details.name,
            current_working_directory: details.current_working_directory,
            root_directory: details.root_directory,
            environment: params.include_environment.then_some(details.environment),
            environment_variable_names: names,
        })
    }
}
