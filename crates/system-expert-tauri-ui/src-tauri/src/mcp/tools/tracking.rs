//! Starting, listing, and deleting tracking sessions. See `tracking_archive` for reading one
//! back once it's ended.

use rmcp::{
    handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
    ErrorData as McpError,
};

use super::helpers::{blocking, db_error, json_ok, tool_error};
use crate::mcp::SystemExpertMcp;

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StartTrackingParams {
    /// A label for the session, shown in the app's Track page.
    pub name: String,
    /// The process ids to track together as one session.
    pub pids: Vec<i64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListTrackedParams {
    /// Only return sessions with this exact name.
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TrackedIdParams {
    /// The tracking session id.
    pub id: i64,
}

#[tool_router(vis = "pub(crate)", router = tool_router_tracking)]
impl SystemExpertMcp {
    #[tool(
        description = "Start a named tracking session over one or more pids. The session and \
                       its recorded samples appear on the app's Track page."
    )]
    async fn start_tracking(
        &self,
        Parameters(params): Parameters<StartTrackingParams>,
    ) -> Result<CallToolResult, McpError> {
        let tracking = self.tracking.clone();
        let tracked =
            blocking(move || tracking.start(&params.name, &params.pids).map_err(db_error)).await?;
        json_ok(&tracked)
    }

    #[tool(description = "List tracking sessions, most recently started first.")]
    async fn list_tracked_processes(
        &self,
        Parameters(params): Parameters<ListTrackedParams>,
    ) -> Result<CallToolResult, McpError> {
        let tracking = self.tracking.clone();
        let sessions =
            blocking(move || tracking.list(params.name.as_deref()).map_err(db_error)).await?;
        json_ok(&sessions)
    }

    #[tool(description = "Delete a tracking session and its archive file.")]
    async fn delete_tracked_process(
        &self,
        Parameters(params): Parameters<TrackedIdParams>,
    ) -> Result<CallToolResult, McpError> {
        let tracking = self.tracking.clone();
        let id = params.id;
        let deleted = blocking(move || {
            let deleted = tracking.delete(id).map_err(db_error)?;
            if let Some(tracked) = &deleted {
                if let Some(path) = &tracked.archive_path {
                    let _ = std::fs::remove_file(path);
                }
            }
            Ok(deleted.is_some())
        })
        .await?;

        if deleted {
            json_ok(&serde_json::json!({ "deleted": true, "id": id }))
        } else {
            Ok(tool_error(format!("no tracking session with id {id}")))
        }
    }
}
