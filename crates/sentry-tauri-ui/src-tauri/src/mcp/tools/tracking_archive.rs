//! Ending a tracking session (archiving it to disk) and reading that archive back.

use rmcp::{
    ErrorData as McpError, handler::server::wrapper::Parameters, model::CallToolResult, tool,
    tool_router,
};
use tauri::Manager;

use super::helpers::{blocking, default_max_points, json_ok, tool_error};
use super::tracking::TrackedIdParams;
use crate::archive;
use crate::mcp::SentryMcp;
use crate::mcp::types::downsample_process;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TrackedArchiveParams {
    /// The tracking session id.
    pub id: i64,
    /// Maximum points per tracked pid (default 120).
    #[serde(default = "default_max_points")]
    pub max_points: usize,
}

#[tool_router(vis = "pub(crate)", router = tool_router_tracking_archive)]
impl SentryMcp {
    #[tool(
        description = "End a tracking session and archive its samples to disk so they stay \
                       readable after history is pruned. Idempotent."
    )]
    async fn end_tracking(
        &self,
        Parameters(params): Parameters<TrackedIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let data_dir = self
            .app
            .path()
            .app_data_dir()
            .map_err(|e| McpError::internal_error(format!("no app data dir: {e}"), None))?;
        let tracking = self.tracking.clone();
        let history = self.history.clone();
        let id = params.id;

        let result = blocking(move || {
            Ok(archive::end_and_archive(&tracking, &history, &data_dir, id))
        })
        .await?;

        match result {
            Ok(tracked) => json_ok(&tracked),
            Err(message) => Ok(tool_error(message)),
        }
    }

    #[tool(
        description = "Read a finished session's archived samples. Bucket-averaged down to \
                       max_points per tracked pid."
    )]
    async fn get_tracked_archive(
        &self,
        Parameters(params): Parameters<TrackedArchiveParams>,
    ) -> Result<CallToolResult, McpError> {
        let tracking = self.tracking.clone();
        let (id, max_points) = (params.id, params.max_points);
        let result = blocking(move || Ok(archive::read_archive(&tracking, id))).await?;

        match result {
            Ok(Some(mut archive)) => {
                archive.samples = downsample_process(archive.samples, max_points);
                json_ok(&archive)
            }
            Ok(None) => Ok(tool_error(format!(
                "tracking session {id} has no archive — it may still be active, or may not exist"
            ))),
            Err(message) => Ok(tool_error(message)),
        }
    }
}
