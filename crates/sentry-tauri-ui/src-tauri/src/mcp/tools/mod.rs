//! The MCP tool surface over `sentry-core`.
//!
//! Grouped one file per feature area — `system`, `processes`, `history`, `tracking`, `chat`,
//! `agent_config`, `action` — each owning its own parameter/response types and the `#[tool]`
//! methods for that area. [`build_tool_router`] merges their independently generated tool
//! routers into the one this server actually serves.
//!
//! Two rules run through every tool here:
//!
//! 1. **Everything blocking goes through [`helpers::blocking`].** `sentry-core` is entirely
//!    synchronous — `Monitor::snapshot` refreshes the whole process table, and every store
//!    method holds a `std::sync::Mutex` across SQLite I/O. Calling those directly from an
//!    async tool body would park an axum worker thread for the duration.
//! 2. **Results are shaped for a context window,** not for a UI table. See
//!    [`super::types`] for the projections and downsampling that implements.

mod action;
mod agent_config;
mod chat;
mod chat_replies;
mod helpers;
mod history;
mod process_details;
mod processes;
mod system;
mod tracking;
mod tracking_archive;

// Re-exported at the `tools::` level, matching where they lived before this module was split
// up — nothing in this crate currently names them by that path (each is used only inside the
// file that defines it), so rustc reads the re-export itself as unused. It's the module's
// public surface, not dead code.
#[allow(unused_imports)]
pub use action::EVENT_PROCESSES_CHANGED;
#[allow(unused_imports)]
pub use chat_replies::{EVENT_CHAT_UPDATED, EVENT_THINKING};

use rmcp::handler::server::router::tool::ToolRouter;

use super::SentryMcp;

/// Merges every feature area's independently generated `#[tool_router]` into one.
///
/// Each area's macro-generated function has its own name (`tool_router_system`,
/// `tool_router_chat`, …) precisely so they can all coexist as inherent methods on `SentryMcp`
/// and be combined here, rather than one `impl` block growing to hold every tool this server
/// exposes.
pub(super) fn build_tool_router() -> ToolRouter<SentryMcp> {
    SentryMcp::tool_router_system()
        + SentryMcp::tool_router_processes()
        + SentryMcp::tool_router_process_details()
        + SentryMcp::tool_router_history()
        + SentryMcp::tool_router_tracking()
        + SentryMcp::tool_router_tracking_archive()
        + SentryMcp::tool_router_chat()
        + SentryMcp::tool_router_chat_replies()
        + SentryMcp::tool_router_agent_config()
        + SentryMcp::tool_router_action()
}
