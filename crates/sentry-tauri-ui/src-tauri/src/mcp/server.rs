//! Binding the MCP server to loopback and telling the agent where to find it.

use std::net::SocketAddr;
use std::sync::Arc;

use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use sentry_core::{ChatStore, HistoryStore, McpUsageStore, TrackingStore};
use serde::Serialize;
use tauri::AppHandle;
use tokio::net::TcpListener;

use super::events::{self, EventBus};
use super::SentryMcp;
use tauri::Manager;

/// The model currently selected in the app, if agent state is available.
fn app_model(app: &AppHandle) -> Option<String> {
    app.try_state::<crate::agent::AgentState>()
        .map(|state| state.lock().model().to_string())
}

/// The one port this server ever uses.
///
/// Deliberately fixed with no fallback: clients — the bundled Python agent, and any MCP
/// client the user points at it — hardcode this address, so quietly binding somewhere else
/// when it's occupied would leave them all failing to connect against a server that looks
/// healthy. Failing to bind is reported instead, which is a diagnosable state.
pub const PORT: u16 = 8765;
const MCP_PATH: &str = "/mcp";
const EVENTS_PATH: &str = "/events";

/// Where the running server can be reached. Returned to the frontend by `get_mcp_status`.
#[derive(Debug, Clone, Serialize)]
pub struct McpStatus {
    pub running: bool,
    pub url: String,
    pub port: u16,
    pub pid: u32,
    pub tool_count: usize,
}

/// The MCP endpoint's address.
pub fn url() -> String {
    format!("http://127.0.0.1:{PORT}{MCP_PATH}")
}

/// The event WebSocket's address.
pub fn events_url() -> String {
    format!("ws://127.0.0.1:{PORT}{EVENTS_PATH}")
}

/// Binds the MCP server and spawns it on Tauri's async runtime.
///
/// Returns once the listener is bound, so a caller that gets `Ok` knows the server is
/// already accepting connections rather than about to fail.
pub async fn start(
    app: AppHandle,
    history: Arc<HistoryStore>,
    chat: Arc<ChatStore>,
    tracking: Arc<TrackingStore>,
    usage: Arc<McpUsageStore>,
    bus: EventBus,
    initial_model: String,
) -> std::io::Result<McpStatus> {
    let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], PORT)))
        .await
        .inspect_err(|e| {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                eprintln!(
                    "sentry-mcp: port {PORT} is already in use — another Sentry instance is \
                     probably running. This instance will have no MCP server or agent."
                );
            }
        })?;

    let app_for_events = app.clone();
    let handler = SentryMcp::new(history, chat, tracking, usage, app);

    // rmcp calls this factory once per session, so the handler must be cheap to clone —
    // every field in SentryMcp is an Arc or an AppHandle for exactly this reason.
    let service = StreamableHttpService::new(
        move || Ok(handler.clone()),
        Arc::new(LocalSessionManager::default()),
        // The default config restricts the inbound `Host` header to loopback, which is the
        // DNS-rebinding protection we want for a local server. Left as-is deliberately.
        StreamableHttpServerConfig::default(),
    );

    let router = axum::Router::new().nest_service(MCP_PATH, service).route(
        EVENTS_PATH,
        axum::routing::any(move |ws| {
            // Each new connection is greeted with the model currently selected, so an
            // agent is correct from its first frame without having to ask.
            let model = app_model(&app_for_events).unwrap_or_else(|| initial_model.clone());
            events::handler(
                ws,
                bus.clone(),
                events::Event::AgentConfig { model },
                app_for_events.clone(),
            )
        }),
    );

    tauri::async_runtime::spawn(async move {
        // `with_connect_info` is what makes `ConnectInfo<SocketAddr>` available inside a
        // tool call's `RequestContext.extensions` (via the HTTP `Parts` rmcp injects there)
        // — see `SentryMcp::call_tool`, which resolves it to a pid for usage logging.
        let make_service = router.into_make_service_with_connect_info::<SocketAddr>();
        if let Err(e) = axum::serve(listener, make_service).await {
            eprintln!("sentry-mcp: server stopped: {e}");
        }
    });

    Ok(McpStatus {
        running: true,
        url: url(),
        port: PORT,
        pid: std::process::id(),
        tool_count: SentryMcp::tool_count(),
    })
}
