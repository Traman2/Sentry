//! Listing and finding processes. See `process_details` for inspecting one.

use rmcp::{
    ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult, tool,
    tool_router,
};

use super::helpers::{blocking, json_ok};
use crate::mcp::SentryMcp;
use crate::mcp::types::{ProcessPage, ProcessSort, page_processes};

fn default_limit() -> usize {
    20
}
fn default_find_limit() -> usize {
    10
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListProcessesParams {
    /// Sort order. Defaults to highest CPU first.
    #[serde(default)]
    pub sort_by: ProcessSort,
    /// Maximum processes to return (default 20). There are usually several hundred running;
    /// asking for all of them will not fit in a useful response.
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Number of processes to skip, for paging through the sorted list.
    #[serde(default)]
    pub offset: usize,
    /// Case-insensitive substring match on the process name.
    #[serde(default)]
    pub name_contains: Option<String>,
    /// Exact (case-insensitive) owning username.
    #[serde(default)]
    pub user: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FindProcessesParams {
    /// Case-insensitive substring of the process name, e.g. "chrome".
    pub name: String,
    /// Maximum matches to return (default 10).
    #[serde(default = "default_find_limit")]
    pub limit: usize,
}

#[tool_router(vis = "pub(crate)", router = tool_router_processes)]
impl SentryMcp {
    #[tool(
        description = "List running processes, sorted and paginated. Defaults to the 20 \
                       highest-CPU processes. Use name_contains to filter, offset to page."
    )]
    async fn list_processes(
        &self,
        Parameters(params): Parameters<ListProcessesParams>,
    ) -> Result<CallToolResult, McpError> {
        let monitor = self.monitor.clone();
        let page: ProcessPage = blocking(move || {
            let mut monitor = monitor.lock().unwrap_or_else(|e| e.into_inner());
            let snapshot = monitor.snapshot();
            Ok(page_processes(
                &snapshot.processes,
                params.sort_by,
                params.name_contains.as_deref(),
                params.user.as_deref(),
                params.limit,
                params.offset,
            ))
        })
        .await?;
        json_ok(&page)
    }

    #[tool(
        description = "Find processes whose name contains a substring. A convenience wrapper \
                       over list_processes for 'is X running' and 'what pid is X' questions."
    )]
    async fn find_processes(
        &self,
        Parameters(params): Parameters<FindProcessesParams>,
    ) -> Result<CallToolResult, McpError> {
        let monitor = self.monitor.clone();
        let page = blocking(move || {
            let mut monitor = monitor.lock().unwrap_or_else(|e| e.into_inner());
            let snapshot = monitor.snapshot();
            Ok(page_processes(
                &snapshot.processes,
                ProcessSort::Cpu,
                Some(&params.name),
                None,
                params.limit,
                0,
            ))
        })
        .await?;
        json_ok(&page)
    }
}
