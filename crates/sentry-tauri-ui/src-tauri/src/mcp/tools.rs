//! The MCP tool surface over `sentry-core`.
//!
//! Two rules run through every tool here:
//!
//! 1. **Everything blocking goes through [`blocking`].** `sentry-core` is entirely
//!    synchronous — `Monitor::snapshot` refreshes the whole process table, and every store
//!    method holds a `std::sync::Mutex` across SQLite I/O. Calling those directly from an
//!    async tool body would park an axum worker thread for the duration.
//! 2. **Results are shaped for a context window,** not for a UI table. See
//!    [`super::types`] for the projections and downsampling that implements.

use std::time::Duration;

use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    tool, tool_router,
};
use serde::Serialize;
use tauri::{Emitter, Manager};

use super::SentryMcp;
use super::types::{
    ProcessPage, ProcessSort, downsample_process, downsample_system, kill_guard, page_processes,
};
use crate::archive;

/// Emitted when an agent changes something the UI is displaying, so the frontend can refresh
/// rather than wait out its polling interval.
pub const EVENT_CHAT_UPDATED: &str = "mcp://chat-updated";
pub const EVENT_PROCESSES_CHANGED: &str = "mcp://processes-changed";
/// A step the agent took on the way to an answer. Deliberately not written to the database:
/// these describe work in progress, and once the reply lands they are scaffolding. Keeping
/// them purely in flight means the final answer replaces them with no cleanup, and a
/// transcript reopened later shows the answer rather than a log of how it was reached.
pub const EVENT_THINKING: &str = "mcp://thinking";

fn default_limit() -> usize {
    20
}
fn default_find_limit() -> usize {
    10
}
fn default_max_points() -> usize {
    120
}
fn default_since_secs() -> u64 {
    3600
}

// ---------------------------------------------------------------------------
// Parameter types
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SystemSummaryParams {
    /// Include a per-core CPU breakdown. Off by default — on a many-core machine this is the
    /// bulk of the response and is rarely what the question needs.
    #[serde(default)]
    pub include_per_core: bool,
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ProcessDetailsParams {
    /// The process id to inspect.
    pub pid: u32,
    /// Return full `KEY=VALUE` environment strings instead of just variable names. Off by
    /// default because a process's environment routinely holds API tokens and credentials.
    #[serde(default)]
    pub include_environment: bool,
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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TrackedArchiveParams {
    /// The tracking session id.
    pub id: i64,
    /// Maximum points per tracked pid (default 120).
    #[serde(default = "default_max_points")]
    pub max_points: usize,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ChatSpaceIdParams {
    /// The chat space id.
    pub id: i64,
}

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

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct KillProcessParams {
    /// The process id to terminate.
    pub pid: u32,
    /// The process name you believe this pid belongs to. Strongly recommended: pids are
    /// recycled by the OS, and passing this makes the call refuse rather than terminate a
    /// different process that inherited the number.
    #[serde(default)]
    pub expect_name: Option<String>,
    /// Override the refusal of critical OS processes and of Sentry itself. Almost never
    /// correct.
    #[serde(default)]
    pub force: bool,
}

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

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

#[derive(Serialize)]
struct TimelineResponse<T> {
    since_secs: u64,
    /// How many samples the store actually held before downsampling — the gap between this
    /// and `returned` is how much resolution was traded away.
    total_samples: usize,
    returned: usize,
    points: Vec<T>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Runs a blocking `sentry-core` call on the blocking pool.
async fn blocking<T, F>(f: F) -> Result<T, McpError>
where
    F: FnOnce() -> Result<T, McpError> + Send + 'static,
    T: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| McpError::internal_error(format!("worker task failed: {e}"), None))?
}

/// Serializes `value` as the tool's text result.
fn json_ok<T: Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let text = serde_json::to_string_pretty(value)
        .map_err(|e| McpError::internal_error(format!("failed to serialize result: {e}"), None))?;
    Ok(CallToolResult::success(vec![ContentBlock::text(text)]))
}

/// A tool-level failure the caller should read, as opposed to `Err(McpError)`, which MCP
/// clients render opaquely.
fn tool_error(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.into())])
}

/// Generic over the error type so this crate doesn't need a direct `rusqlite` dependency
/// just to name `sentry-core`'s error.
fn db_error<E: std::fmt::Display>(e: E) -> McpError {
    McpError::internal_error(format!("database error: {e}"), None)
}

#[tool_router(vis = "pub(crate)")]
impl SentryMcp {
    // -- System state -------------------------------------------------------

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

    // -- History ------------------------------------------------------------

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

    // -- Tracking -----------------------------------------------------------

    #[tool(
        description = "Start a named tracking session over one or more pids. The session and \
                       its recorded samples appear on the app's Track page."
    )]
    async fn start_tracking(
        &self,
        Parameters(params): Parameters<StartTrackingParams>,
    ) -> Result<CallToolResult, McpError> {
        let tracking = self.tracking.clone();
        let tracked = blocking(move || {
            tracking
                .start(&params.name, &params.pids)
                .map_err(db_error)
        })
        .await?;
        json_ok(&tracked)
    }

    #[tool(description = "List tracking sessions, most recently started first.")]
    async fn list_tracked_processes(
        &self,
        Parameters(params): Parameters<ListTrackedParams>,
    ) -> Result<CallToolResult, McpError> {
        let tracking = self.tracking.clone();
        let sessions = blocking(move || {
            tracking.list(params.name.as_deref()).map_err(db_error)
        })
        .await?;
        json_ok(&sessions)
    }

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

    // -- Chat ---------------------------------------------------------------

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

    // -- Agent configuration ------------------------------------------------

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
            "events_url": super::server::events_url(),
        }))
    }

    // -- Action -------------------------------------------------------------

    #[tool(
        description = "Terminate a process. This is irreversible and affects the user's real \
                       machine — look the process up first and pass expect_name from what you \
                       saw, so a recycled pid cannot make this kill the wrong process. \
                       Critical OS processes and Sentry itself are refused unless force is set."
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
