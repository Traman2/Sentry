//! Whole-machine state: the summary and the per-category listings below it.

use rmcp::{
    handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
    ErrorData as McpError,
};

use super::helpers::{blocking, json_ok};
use crate::mcp::SentryMcp;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SystemSummaryParams {
    /// Include a per-core CPU breakdown. Off by default — on a many-core machine this is the
    /// bulk of the response and is rarely what the question needs.
    #[serde(default)]
    pub include_per_core: bool,
}

#[tool_router(vis = "pub(crate)", router = tool_router_system)]
impl SentryMcp {
    #[tool(
        description = "Whole-machine summary: OS, uptime, CPU usage, memory and swap. Start \
                       here for 'how is this machine doing' questions."
    )]
    async fn get_system_summary(
        &self,
        Parameters(params): Parameters<SystemSummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        let monitor = self.monitor.clone();
        let summary = blocking(move || {
            let mut monitor = monitor.lock().unwrap_or_else(|e| e.into_inner());
            let mut summary = monitor.snapshot().system;
            if !params.include_per_core {
                summary.per_core_usage = Vec::new();
            }
            Ok(summary)
        })
        .await?;
        json_ok(&summary)
    }

    #[tool(description = "Storage volumes: capacity, used and free space per mount point.")]
    async fn list_disks(&self) -> Result<CallToolResult, McpError> {
        let monitor = self.monitor.clone();
        let disks = blocking(move || {
            let mut monitor = monitor.lock().unwrap_or_else(|e| e.into_inner());
            Ok(monitor.snapshot().disks)
        })
        .await?;
        json_ok(&disks)
    }

    #[tool(
        description = "Network interfaces with current throughput. Note that the OS does not \
                       attribute network traffic to individual processes, so this is \
                       system-wide only."
    )]
    async fn list_networks(&self) -> Result<CallToolResult, McpError> {
        let monitor = self.monitor.clone();
        let networks = blocking(move || {
            let mut monitor = monitor.lock().unwrap_or_else(|e| e.into_inner());
            Ok(monitor.snapshot().networks)
        })
        .await?;
        json_ok(&networks)
    }

    #[tool(
        description = "Hardware temperature sensors, where the platform exposes them. Often \
                       empty on Windows."
    )]
    async fn list_components(&self) -> Result<CallToolResult, McpError> {
        let monitor = self.monitor.clone();
        let components = blocking(move || {
            let mut monitor = monitor.lock().unwrap_or_else(|e| e.into_inner());
            Ok(monitor.snapshot().components)
        })
        .await?;
        json_ok(&components)
    }
}
