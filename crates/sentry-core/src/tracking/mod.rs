//! "Track" sessions: a user-initiated watch over a fixed set of pids (a single
//! process, or every pid backing an app), started from the Resource Monitor's
//! Track button.
//!
//! Session bookkeeping (which pids, since when, active/ended, where its archive
//! lives) is stored here in SQLite via [`TrackingStore`], mirroring
//! [`super::chat`]'s storage approach. The actual CPU/memory samples come from
//! [`super::history::HistoryStore`], which already records every process on a
//! fixed interval independent of tracking — so a track session just queries that
//! existing timeline for its pids rather than running its own collector. When a
//! session ends, its samples are frozen to a JSON file (a [`TrackedArchive`]) so
//! they remain viewable after `HistoryStore`'s retention window prunes the live
//! rows.

mod models;
mod store;

pub use models::{TrackedArchive, TrackedProcess};
pub use store::TrackingStore;

#[cfg(test)]
mod tests;
