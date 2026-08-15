//! Slim projections, downsampling, and kill guardrails for the MCP tool surface.
//!
//! Nothing here talks to `sentry-core` — these are pure shaping functions, kept separate
//! so they stay testable without a running `Monitor` or a SQLite file.
//!
//! ## Why projections exist
//!
//! `sentry-core`'s types are built for a UI table: every numeric field is paired with a
//! pre-formatted `_display` string, and a [`sentry_core::SystemSnapshot`] carries every
//! process on the machine. That's the right shape for a React grid and the wrong shape for
//! a tool result — a typical snapshot is several hundred processes × ~30 fields, which is
//! well over 100k tokens of JSON and would swamp the context of whatever agent called it.
//! Every list-returning tool therefore projects down to the handful of fields an agent
//! actually reasons over, and paginates.

mod downsample;
mod guard;
mod process;

pub use downsample::{downsample_process, downsample_system};
pub use guard::kill_guard;
pub use process::{ProcessPage, ProcessSort, page_processes};
