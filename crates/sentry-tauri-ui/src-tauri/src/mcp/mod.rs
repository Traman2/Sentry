//! The MCP server, hosted inside the Tauri process.
//!
//! # Why in-process
//!
//! An MCP server for this app could have been its own binary linking `sentry-core`, and
//! originally was. Running it inside the desktop app instead means the tools an agent calls
//! read and mutate *the same* stores the on-screen UI is rendering — an agent that ends a
//! tracking session or posts a chat message is changing the state the user is looking at,
//! with no second process and no synchronization between two copies of the truth.
//!
//! That forces the transport: the agent is a separate (Python) process, so stdio is
//! unavailable — Tauri owns this process's stdio and there is no child to speak through.
//! The server is therefore Streamable HTTP bound to loopback. See [`server`].
//!
//! # State sharing
//!
//! The three SQLite-backed stores are shared with the Tauri command layer via `Arc`; they're
//! `Send + Sync` behind an internal `Mutex<Connection>`, so agent and UI genuinely see one
//! database. [`sentry_core::Monitor`] is deliberately *not* shared — see [`SentryMcp`].

pub mod events;
pub mod server;
pub mod tools;
pub mod types;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::extract::ConnectInfo;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::tool::ToolCallContext,
    model::{
        CallToolRequestParams, CallToolResponse, Implementation, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool_handler,
};
use sentry_core::{
    CallStatus, ChatStore, ClientIdentity, HistoryStore, McpUsageStore, Monitor, TrackingStore,
};
use tauri::{AppHandle, Emitter};
use tools::build_tool_router;

pub use events::{Event, EventBus};
pub use server::{McpStatus, start};

/// Emitted whenever an MCP tool call is recorded, so the frontend's usage panel can refresh
/// rather than wait out a polling interval — same mechanism as `EVENT_CHAT_UPDATED`.
pub const EVENT_MCP_USAGE_UPDATED: &str = "mcp://usage-updated";

/// The MCP tool handler. Cheap to clone — [`rmcp`] builds one per session via the service
/// factory, and every field is either an `Arc` or an `AppHandle`.
#[derive(Clone)]
pub struct SentryMcp {
    /// The MCP server's **own** `Monitor`, deliberately not the one behind the UI's
    /// `MonitorState`.
    ///
    /// `sysinfo` derives CPU percentages and disk/network throughput as deltas since *that
    /// instance's* previous refresh. Sharing one `Monitor` would mean an agent poll landing
    /// between two UI polls shortens the UI's delta window, visibly distorting the numbers
    /// on screen — and the reverse, an agent seeing figures scaled to whenever the UI last
    /// happened to tick. Two instances cost a little memory and give each caller a correct,
    /// independent baseline.
    pub(crate) monitor: Arc<Mutex<Monitor>>,
    pub(crate) history: Arc<HistoryStore>,
    pub(crate) chat: Arc<ChatStore>,
    pub(crate) tracking: Arc<TrackingStore>,
    /// Which MCP clients have called which tools, and on what OS process — see
    /// [`SentryMcp::call_tool`], the only place this is written.
    pub(crate) usage: Arc<McpUsageStore>,
    /// Used to resolve the app data dir for tracked-session archives, and to emit events so
    /// the frontend refreshes when an agent changes something.
    pub(crate) app: AppHandle,
    tool_router: ToolRouter<SentryMcp>,
}

impl SentryMcp {
    pub fn new(
        history: Arc<HistoryStore>,
        chat: Arc<ChatStore>,
        tracking: Arc<TrackingStore>,
        usage: Arc<McpUsageStore>,
        app: AppHandle,
    ) -> Self {
        Self {
            monitor: Arc::new(Mutex::new(Monitor::new())),
            history,
            chat,
            tracking,
            usage,
            app,
            tool_router: build_tool_router(),
        }
    }

    /// How many tools this server exposes — surfaced to the UI through `get_mcp_status`.
    pub fn tool_count() -> usize {
        build_tool_router().list_all().len()
    }

    /// Records who called `tool_name` and whether the call succeeded at the protocol level,
    /// then tells the frontend to refresh its usage panel.
    ///
    /// Best-effort throughout: a failure to resolve a pid, or to write the row, is logged to
    /// stderr and otherwise swallowed. Usage logging must never turn a working tool call into
    /// a failed one, which is also why this is awaited *after* `result` is already computed —
    /// nothing here can affect what the caller gets back, only how long it takes to get it.
    async fn log_tool_call(
        &self,
        client_info: Option<Implementation>,
        remote_port: Option<u16>,
        tool_name: String,
        params_json: Option<String>,
        result: &Result<CallToolResponse, McpError>,
        elapsed: Duration,
    ) {
        let (status, error_message) = match result {
            Ok(_) => (CallStatus::Ok, None),
            Err(e) => (CallStatus::ProtocolError, Some(e.message.to_string())),
        };
        let duration_ms = elapsed.as_millis() as i64;

        let monitor = self.monitor.clone();
        let usage = self.usage.clone();

        let recorded = tokio::task::spawn_blocking(move || {
            // Both ends of a loopback TCP connection are local sockets, so the OS's own TCP
            // table can tell us which process holds the client's end of it — see
            // `pid_for_loopback_client` for exactly how.
            let pid = remote_port
                .and_then(|port| sentry_core::monitor::pid_for_loopback_client(server::PORT, port));

            let (process_name, process_command) = pid
                .and_then(|pid| {
                    monitor
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .process_identity(pid)
                })
                .map(|identity| (Some(identity.name), Some(identity.command)))
                .unwrap_or((None, None));

            let identity = ClientIdentity {
                protocol_name: client_info.as_ref().map(|i| i.name.clone()),
                protocol_version: client_info.as_ref().map(|i| i.version.clone()),
                pid,
                process_name,
                process_command,
            };

            usage.record_call(
                &identity,
                &tool_name,
                params_json.as_deref(),
                status,
                duration_ms,
                error_message.as_deref(),
            )
        })
        .await;

        match recorded {
            Ok(Ok((client, _call))) => {
                let _ = self.app.emit(EVENT_MCP_USAGE_UPDATED, client.id);
            }
            Ok(Err(e)) => eprintln!("sentry-mcp: failed to record tool-call usage: {e}"),
            Err(e) => eprintln!("sentry-mcp: usage-logging task failed: {e}"),
        }
    }
}

// `router = self.tool_router` rather than the macro's default of `Self::tool_router()`: the
// default rebuilds and re-registers all 22 tools on every single `list_tools` and
// `call_tool`. Pointing at the field builds the router once per session instead.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for SentryMcp {
    // Defined by hand, rather than left to `#[tool_handler]`, so every tool call can be
    // logged around the same dispatch the macro would otherwise generate — `#[tool_handler]`
    // only fills in methods not already present on this impl, so `list_tools`/`get_tool`/
    // `get_info` below are still auto-generated as usual.
    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, McpError> {
        let started = Instant::now();
        let client_info = context.peer.peer_info().map(|info| info.client_info.clone());
        let remote_port = context
            .extensions
            .get::<axum::http::request::Parts>()
            .and_then(|parts| parts.extensions.get::<ConnectInfo<SocketAddr>>())
            .map(|connect_info| connect_info.0.port());
        let tool_name = request.name.to_string();
        let params_json = request
            .arguments
            .as_ref()
            .and_then(|args| serde_json::to_string(args).ok());

        let tcc = ToolCallContext::new(self, request, context);
        let result = self.tool_router.call(tcc).await;

        self.log_tool_call(client_info, remote_port, tool_name, params_json, &result, started.elapsed())
            .await;

        result
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new("sentry-mcp", env!("CARGO_PKG_VERSION")))
            .with_instructions(
                "Sentry exposes the live state of this machine: processes, CPU/memory/disk/\
                 network, recorded history, tracked sessions, and the desktop app's chat.\n\n\
                 Start with `get_system_summary` for the whole-machine picture and \
                 `list_processes` for what's running — it is paginated and sorted by CPU by \
                 default, so ask for what you need rather than everything. History and \
                 archive tools downsample to `max_points`; raise it only if you actually need \
                 the resolution.\n\n\
                 `kill_process` terminates a real process on the user's machine and cannot be \
                 undone. Look the process up first and pass `expect_name` from what you saw, \
                 so a recycled pid can't make you kill the wrong thing. Critical OS processes \
                 are refused unless `force` is set.\n\n\
                 `post_assistant_message` writes into a chat space in the app's UI, where the \
                 user will read it."
                    .to_string(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_count_matches_every_registered_tool_router() {
        // `build_tool_router` merges one `ToolRouter` per `tools/*.rs` file by name; a
        // collision (e.g. two files defining a tool of the same name, or a file whose router
        // was never added to the sum) drops a tool silently rather than failing to compile —
        // `ToolRouter::merge` just overwrites. This pins the expected count so that kind of
        // regression fails a test instead of only showing up as a tool missing from an
        // agent's tool list.
        assert_eq!(SentryMcp::tool_count(), 22);
    }
}
