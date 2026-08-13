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

use std::sync::{Arc, Mutex};

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool_handler,
};
use sentry_core::{ChatStore, HistoryStore, Monitor, TrackingStore};
use tauri::AppHandle;

pub use events::{Event, EventBus};
pub use server::{McpStatus, start};

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
        app: AppHandle,
    ) -> Self {
        Self {
            monitor: Arc::new(Mutex::new(Monitor::new())),
            history,
            chat,
            tracking,
            app,
            tool_router: Self::tool_router(),
        }
    }

    /// How many tools this server exposes — surfaced to the UI through `get_mcp_status`.
    pub fn tool_count() -> usize {
        Self::tool_router().list_all().len()
    }
}

// `router = self.tool_router` rather than the macro's default of `Self::tool_router()`: the
// default rebuilds and re-registers all ~19 tools on every single `list_tools` and
// `call_tool`. Pointing at the field builds the router once per session instead.
#[tool_handler(router = self.tool_router)]
impl ServerHandler for SentryMcp {
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
