//! Row types returned by [`super::TrackingStore`].

use serde::{Deserialize, Serialize};

use crate::history::ProcessSamplePoint;

/// A "Track" session over a fixed set of pids, started when the user clicks
/// Track on a process/app row in the Resource Monitor.
#[derive(Debug, Clone, Serialize)]
pub struct TrackedProcess {
    pub id: i64,
    /// The process/app name shown in the Resource Monitor at the time tracking
    /// started.
    pub name: String,
    /// The exact pids being tracked, captured once at track-start — an app is
    /// this fixed set, not a live re-grouping by name.
    pub pids: Vec<i64>,
    pub started_at_ms: i64,
    pub ended_at_ms: Option<i64>,
    /// `"active"` while still live, `"ended"` once stopped (manually or because
    /// every tracked pid disappeared).
    pub status: String,
    /// Path to the archived sample dump on disk, set once tracking ends.
    pub archive_path: Option<String>,
}

/// The file written to `archive_path` when a [`TrackedProcess`] ends — a frozen
/// copy of its samples so the Details View can still show the feed after the
/// live `process_samples` rows age out of [`crate::history::DEFAULT_RETENTION`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedArchive {
    pub id: i64,
    pub name: String,
    pub pids: Vec<i64>,
    pub started_at_ms: i64,
    pub ended_at_ms: i64,
    pub samples: Vec<ProcessSamplePoint>,
}
