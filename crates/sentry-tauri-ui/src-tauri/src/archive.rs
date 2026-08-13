//! Ending a tracked session and archiving its samples.
//!
//! This lives outside both the Tauri command layer and the MCP tool layer because *both*
//! need it and the archive write is the part that must not be skipped: once a session ends,
//! its `process_samples` rows keep ageing out at [`sentry_core::DEFAULT_RETENTION`], so a
//! session ended without archiving shows an empty feed in the Details View a day later.

use std::path::Path;

use sentry_core::{HistoryStore, TrackedArchive, TrackedProcess, TrackingStore};

/// Ends the tracking session `id` and, the first time, writes its samples to
/// `<data_dir>/tracked_archives/<id>.json`.
///
/// Idempotent in both halves: `TrackingStore::end` only updates rows still marked active,
/// and a session that already has an `archive_path` is returned untouched rather than
/// re-archived from history that has since been pruned.
pub fn end_and_archive(
    tracking: &TrackingStore,
    history: &HistoryStore,
    data_dir: &Path,
    id: i64,
) -> Result<TrackedProcess, String> {
    let ended = tracking
        .end(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "tracked process not found".to_string())?;

    if ended.archive_path.is_some() {
        return Ok(ended);
    }

    let pids: Vec<u32> = ended.pids.iter().map(|&p| p as u32).collect();
    let samples = history
        .process_timeline_for_pids(&pids, sentry_core::DEFAULT_RETENTION)
        .map_err(|e| e.to_string())?;

    let archive = TrackedArchive {
        id: ended.id,
        name: ended.name.clone(),
        pids: ended.pids.clone(),
        started_at_ms: ended.started_at_ms,
        ended_at_ms: ended.ended_at_ms.unwrap_or(ended.started_at_ms),
        samples,
    };

    let archive_dir = data_dir.join("tracked_archives");
    std::fs::create_dir_all(&archive_dir).map_err(|e| e.to_string())?;
    let archive_path = archive_dir.join(format!("{id}.json"));
    let json = serde_json::to_string_pretty(&archive).map_err(|e| e.to_string())?;
    std::fs::write(&archive_path, json).map_err(|e| e.to_string())?;

    tracking
        .set_archive_path(id, &archive_path.to_string_lossy())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "tracked process not found".to_string())
}

/// Reads a session's archive back off disk, or `None` if it was never archived.
pub fn read_archive(
    tracking: &TrackingStore,
    id: i64,
) -> Result<Option<TrackedArchive>, String> {
    let Some(tracked) = tracking.get(id).map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    let Some(path) = tracked.archive_path else {
        return Ok(None);
    };
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content)
        .map(Some)
        .map_err(|e| e.to_string())
}
