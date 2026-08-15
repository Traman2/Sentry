//! Recorded whole-machine and per-process history.

use std::time::Duration;

use rmcp::{
    handler::server::wrapper::Parameters, model::CallToolResult, tool, tool_router,
    ErrorData as McpError,
};
use serde::Serialize;

use super::helpers::{blocking, db_error, default_max_points, json_ok};
use crate::mcp::types::{downsample_process, downsample_system};
use crate::mcp::SentryMcp;

fn default_since_secs() -> u64 {
    3600
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SystemTimelineParams {
    /// How far back to look, in seconds (default 3600). History is retained for 24h.
    #[serde(default = "default_since_secs")]
    pub since_secs: u64,
    /// Maximum points to return (default 120). The series is bucket-averaged down to this,
    /// preserving the shape of the curve.
    #[serde(default = "default_max_points")]
    pub max_points: usize,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProcessTimelineParams {
    /// The process id whose history to return.
    pub pid: u32,
    /// How far back to look, in seconds (default 3600).
    #[serde(default = "default_since_secs")]
    pub since_secs: u64,
    /// Maximum points to return (default 120).
    #[serde(default = "default_max_points")]
    pub max_points: usize,
}

#[derive(Serialize)]
struct TimelineResponse<T> {
    since_secs: u64,
    /// How many samples the store actually held before downsampling — the gap between this
    /// and `returned` is how much resolution was traded away.
    total_samples: usize,
    returned: usize,
    points: Vec<T>,
}

#[tool_router(vis = "pub(crate)", router = tool_router_history)]
impl SentryMcp {
    #[tool(
        description = "Recorded whole-machine history (CPU, memory, swap, network) over a \
                       time window. Bucket-averaged down to max_points."
    )]
    async fn get_system_timeline(
        &self,
        Parameters(params): Parameters<SystemTimelineParams>,
    ) -> Result<CallToolResult, McpError> {
        let history = self.history.clone();
        let since = params.since_secs;
        let max_points = params.max_points;
        let response = blocking(move || {
            let points = history
                .system_timeline(Duration::from_secs(since))
                .map_err(db_error)?;
            let total = points.len();
            let points = downsample_system(points, max_points);
            Ok(TimelineResponse {
                since_secs: since,
                total_samples: total,
                returned: points.len(),
                points,
            })
        })
        .await?;
        json_ok(&response)
    }

    #[tool(
        description = "Recorded history for one process over a time window. Bucket-averaged \
                       down to max_points. Returns no points if the pid was not running \
                       during the window."
    )]
    async fn get_process_timeline(
        &self,
        Parameters(params): Parameters<ProcessTimelineParams>,
    ) -> Result<CallToolResult, McpError> {
        let history = self.history.clone();
        let (pid, since, max_points) = (params.pid, params.since_secs, params.max_points);
        let response = blocking(move || {
            let points = history
                .process_timeline(pid, Duration::from_secs(since))
                .map_err(db_error)?;
            let total = points.len();
            let points = downsample_process(points, max_points);
            Ok(TimelineResponse {
                since_secs: since,
                total_samples: total,
                returned: points.len(),
                points,
            })
        })
        .await?;
        json_ok(&response)
    }
}
