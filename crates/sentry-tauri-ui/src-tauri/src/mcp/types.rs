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

use sentry_core::{ProcessRow, ProcessSamplePoint, SystemSamplePoint};
use serde::Serialize;

/// Windows processes that terminating would take the machine down with them. Killing any of
/// these requires `force: true`, which exists as an escape hatch for a caller that genuinely
/// knows better rather than as a thing to reach for.
const CRITICAL_PROCESS_NAMES: &[&str] = &[
    "system",
    "registry",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
];

/// One process, reduced to what an agent reasons over. Compare
/// [`sentry_core::ProcessRow`]'s 30 fields.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessBrief {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub memory: String,
    pub memory_percent: f32,
    pub user: Option<String>,
    pub status: String,
}

impl From<&ProcessRow> for ProcessBrief {
    fn from(row: &ProcessRow) -> Self {
        Self {
            pid: row.pid,
            name: row.name.clone(),
            cpu_percent: row.cpu_usage_percent,
            memory_bytes: row.memory_bytes,
            memory: row.memory_display.clone(),
            memory_percent: row.memory_percent,
            user: row.user.clone(),
            status: row.status.clone(),
        }
    }
}

/// A page of processes. `total_matched` and `total_processes` are what let an agent tell
/// "these are all of them" from "there are 400 more" without fetching the rest.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessPage {
    pub total_processes: usize,
    pub total_matched: usize,
    pub offset: usize,
    pub returned: usize,
    pub processes: Vec<ProcessBrief>,
}

/// How to order [`ProcessPage`] results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProcessSort {
    /// Highest CPU first — the default, and what "what's slowing my machine down" means.
    Cpu,
    /// Highest resident memory first.
    Memory,
    /// Highest combined disk read+write rate first.
    Disk,
    /// Alphabetical by process name.
    Name,
}

impl Default for ProcessSort {
    fn default() -> Self {
        Self::Cpu
    }
}

/// Filters, sorts, and pages `rows` into a [`ProcessPage`].
pub fn page_processes(
    rows: &[ProcessRow],
    sort: ProcessSort,
    name_contains: Option<&str>,
    user: Option<&str>,
    limit: usize,
    offset: usize,
) -> ProcessPage {
    let needle = name_contains.map(|n| n.to_lowercase());
    let user_needle = user.map(|u| u.to_lowercase());

    let mut matched: Vec<&ProcessRow> = rows
        .iter()
        .filter(|row| {
            needle
                .as_ref()
                .is_none_or(|n| row.name.to_lowercase().contains(n))
        })
        .filter(|row| {
            user_needle.as_ref().is_none_or(|u| {
                row.user
                    .as_ref()
                    .is_some_and(|owner| owner.to_lowercase() == *u)
            })
        })
        .collect();

    match sort {
        // Descending, and `total_cmp` rather than `partial_cmp().unwrap()` so a NaN CPU
        // reading sorts predictably instead of panicking.
        ProcessSort::Cpu => {
            matched.sort_by(|a, b| b.cpu_usage_percent.total_cmp(&a.cpu_usage_percent))
        }
        ProcessSort::Memory => matched.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes)),
        ProcessSort::Disk => matched.sort_by(|a, b| {
            let rate = |r: &ProcessRow| r.disk_read_bytes_per_sec + r.disk_written_bytes_per_sec;
            rate(b).cmp(&rate(a))
        }),
        ProcessSort::Name => {
            matched.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
        }
    }

    let total_matched = matched.len();
    let processes: Vec<ProcessBrief> = matched
        .into_iter()
        .skip(offset)
        .take(limit)
        .map(ProcessBrief::from)
        .collect();

    ProcessPage {
        total_processes: rows.len(),
        total_matched,
        offset,
        returned: processes.len(),
        processes,
    }
}

/// Bucket-averages `points` down to at most `max_points`, preserving chronological order.
///
/// The recorder samples every 2s and retains 24h, so an ungated "show me the last day"
/// query returns ~43,200 points. Averaging into buckets keeps the shape of the curve —
/// which is all an agent needs to answer "was this spiking an hour ago" — at a size that
/// fits in a tool result.
pub fn downsample_system(points: Vec<SystemSamplePoint>, max_points: usize) -> Vec<SystemSamplePoint> {
    bucket(points, max_points, |chunk| {
        let n = chunk.len() as f64;
        SystemSamplePoint {
            // The bucket's last timestamp, so the final point is the most recent reading
            // rather than an average that lands slightly in the past.
            timestamp_ms: chunk[chunk.len() - 1].timestamp_ms,
            cpu_usage_percent: (chunk.iter().map(|p| p.cpu_usage_percent as f64).sum::<f64>() / n)
                as f32,
            used_memory_bytes: (chunk.iter().map(|p| p.used_memory_bytes as f64).sum::<f64>() / n)
                as u64,
            total_memory_bytes: chunk[chunk.len() - 1].total_memory_bytes,
            used_swap_bytes: (chunk.iter().map(|p| p.used_swap_bytes as f64).sum::<f64>() / n)
                as u64,
            network_rx_bytes_per_sec: (chunk
                .iter()
                .map(|p| p.network_rx_bytes_per_sec as f64)
                .sum::<f64>()
                / n) as u64,
            network_tx_bytes_per_sec: (chunk
                .iter()
                .map(|p| p.network_tx_bytes_per_sec as f64)
                .sum::<f64>()
                / n) as u64,
        }
    })
}

/// Bucket-averages process samples down to at most `max_points` **per pid**.
///
/// Grouping by pid first is not optional: a tracked session covers several pids at once, and
/// averaging across a chunk that straddles two different processes would invent readings
/// belonging to neither.
pub fn downsample_process(
    points: Vec<ProcessSamplePoint>,
    max_points: usize,
) -> Vec<ProcessSamplePoint> {
    let mut by_pid: Vec<(u32, Vec<ProcessSamplePoint>)> = Vec::new();
    for point in points {
        match by_pid.iter_mut().find(|(pid, _)| *pid == point.pid) {
            Some((_, group)) => group.push(point),
            None => by_pid.push((point.pid, vec![point])),
        }
    }

    let mut out: Vec<ProcessSamplePoint> = by_pid
        .into_iter()
        .flat_map(|(_, group)| {
            bucket(group, max_points, |chunk| {
                let n = chunk.len() as f64;
                let last = &chunk[chunk.len() - 1];
                ProcessSamplePoint {
                    timestamp_ms: last.timestamp_ms,
                    pid: last.pid,
                    name: last.name.clone(),
                    cpu_usage_percent: (chunk
                        .iter()
                        .map(|p| p.cpu_usage_percent as f64)
                        .sum::<f64>()
                        / n) as f32,
                    memory_bytes: (chunk.iter().map(|p| p.memory_bytes as f64).sum::<f64>() / n)
                        as u64,
                    disk_read_bytes_per_sec: (chunk
                        .iter()
                        .map(|p| p.disk_read_bytes_per_sec as f64)
                        .sum::<f64>()
                        / n) as u64,
                    disk_written_bytes_per_sec: (chunk
                        .iter()
                        .map(|p| p.disk_written_bytes_per_sec as f64)
                        .sum::<f64>()
                        / n) as u64,
                }
            })
        })
        .collect();

    out.sort_by_key(|p| (p.timestamp_ms, p.pid));
    out
}

/// Splits `points` into at most `max_points` contiguous chunks and folds each with `reduce`.
fn bucket<T, F>(points: Vec<T>, max_points: usize, reduce: F) -> Vec<T>
where
    F: Fn(&[T]) -> T,
{
    if max_points == 0 || points.len() <= max_points {
        return points;
    }
    // Ceiling division: with 43,200 points and max 120 this is 360 per bucket, yielding
    // exactly 120 buckets. Rounding down instead would overshoot `max_points`.
    let chunk_size = points.len().div_ceil(max_points);
    points.chunks(chunk_size).map(reduce).collect()
}

/// Checks a kill request against the guardrails, returning `Some(reason)` if it should be
/// refused.
///
/// These live here rather than in `sentry-core` on purpose: the core library stays a neutral
/// mechanism, and policy about what an *agent* is allowed to terminate belongs at the edge
/// that exposes it. `force` overrides everything except pid 0, which is never a real
/// killable process on any supported platform.
pub fn kill_guard(pid: u32, name: Option<&str>, force: bool) -> Option<String> {
    if pid == 0 {
        return Some("pid 0 is not a real process".to_string());
    }

    if force {
        return None;
    }

    if pid == std::process::id() {
        return Some(format!(
            "pid {pid} is Sentry itself — killing it would take this MCP server down with it; \
             pass force: true if that is genuinely what you want"
        ));
    }

    #[cfg(windows)]
    if pid == 4 {
        return Some(
            "pid 4 is the Windows System process — terminating it bugchecks the machine; \
             pass force: true to override"
                .to_string(),
        );
    }

    if let Some(name) = name {
        let lowered = name.to_lowercase();
        if CRITICAL_PROCESS_NAMES.contains(&lowered.as_str()) {
            return Some(format!(
                "{name} is a critical OS process — terminating it will destabilise or reboot the \
                 machine; pass force: true to override"
            ));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sys_point(timestamp_ms: i64, cpu: f32) -> SystemSamplePoint {
        SystemSamplePoint {
            timestamp_ms,
            cpu_usage_percent: cpu,
            used_memory_bytes: 100,
            total_memory_bytes: 1000,
            used_swap_bytes: 0,
            network_rx_bytes_per_sec: 10,
            network_tx_bytes_per_sec: 20,
        }
    }

    fn proc_point(timestamp_ms: i64, pid: u32, cpu: f32) -> ProcessSamplePoint {
        ProcessSamplePoint {
            timestamp_ms,
            pid,
            name: format!("p{pid}"),
            cpu_usage_percent: cpu,
            memory_bytes: 100,
            disk_read_bytes_per_sec: 0,
            disk_written_bytes_per_sec: 0,
        }
    }

    #[test]
    fn downsample_leaves_short_series_untouched() {
        let points = vec![sys_point(1, 10.0), sys_point(2, 20.0)];
        let out = downsample_system(points, 120);
        assert_eq!(out.len(), 2);
        assert_eq!(out[1].cpu_usage_percent, 20.0);
    }

    #[test]
    fn downsample_never_exceeds_max_points() {
        let points: Vec<_> = (0..43_200).map(|i| sys_point(i, 50.0)).collect();
        let out = downsample_system(points, 120);
        assert!(out.len() <= 120, "got {} points", out.len());
    }

    #[test]
    fn downsample_averages_within_a_bucket_and_keeps_the_last_timestamp() {
        let points = vec![
            sys_point(1, 0.0),
            sys_point(2, 100.0),
            sys_point(3, 0.0),
            sys_point(4, 100.0),
        ];
        let out = downsample_system(points, 2);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].cpu_usage_percent, 50.0);
        assert_eq!(out[0].timestamp_ms, 2);
        assert_eq!(out[1].timestamp_ms, 4);
    }

    #[test]
    fn downsample_process_never_averages_across_two_pids() {
        // Interleaved pids: a naive chunker would blend 10.0 and 90.0 into 50.0 for both.
        let points = vec![
            proc_point(1, 100, 10.0),
            proc_point(1, 200, 90.0),
            proc_point(2, 100, 10.0),
            proc_point(2, 200, 90.0),
        ];
        let out = downsample_process(points, 1);

        assert_eq!(out.len(), 2);
        let p100 = out.iter().find(|p| p.pid == 100).unwrap();
        let p200 = out.iter().find(|p| p.pid == 200).unwrap();
        assert_eq!(p100.cpu_usage_percent, 10.0);
        assert_eq!(p200.cpu_usage_percent, 90.0);
    }

    #[test]
    fn kill_guard_always_refuses_pid_zero_even_with_force() {
        assert!(kill_guard(0, None, true).is_some());
    }

    #[test]
    fn kill_guard_refuses_sentry_itself_unless_forced() {
        let me = std::process::id();
        assert!(kill_guard(me, None, false).is_some());
        assert!(kill_guard(me, None, true).is_none());
    }

    #[test]
    fn kill_guard_refuses_critical_names_case_insensitively() {
        assert!(kill_guard(1234, Some("LSASS.EXE"), false).is_some());
        assert!(kill_guard(1234, Some("lsass.exe"), true).is_none());
        assert!(kill_guard(1234, Some("notepad.exe"), false).is_none());
    }

    #[test]
    fn page_processes_reports_totals_independent_of_the_page() {
        let rows: Vec<ProcessRow> = (0..50).map(|i| row(i, "chrome.exe", i as f32)).collect();
        let page = page_processes(&rows, ProcessSort::Cpu, None, None, 10, 0);

        assert_eq!(page.total_processes, 50);
        assert_eq!(page.total_matched, 50);
        assert_eq!(page.returned, 10);
        // Sorted by CPU descending, so the highest pid (highest cpu) leads.
        assert_eq!(page.processes[0].pid, 49);
    }

    #[test]
    fn page_processes_filters_by_name_before_counting_matches() {
        let rows = vec![
            row(1, "chrome.exe", 5.0),
            row(2, "notepad.exe", 1.0),
            row(3, "Chrome.exe", 9.0),
        ];
        let page = page_processes(&rows, ProcessSort::Cpu, Some("chrome"), None, 10, 0);

        assert_eq!(page.total_processes, 3);
        assert_eq!(page.total_matched, 2, "match should be case-insensitive");
        assert_eq!(page.processes[0].pid, 3);
    }

    fn row(pid: u32, name: &str, cpu: f32) -> ProcessRow {
        ProcessRow {
            pid,
            parent_pid: None,
            name: name.to_string(),
            executable_path: None,
            command: String::new(),
            status: "Run".to_string(),
            user: None,
            effective_user: None,
            group_id: None,
            effective_group_id: None,
            session_id: None,
            thread_count: None,
            start_time_unix_secs: 0,
            run_time_secs: 0,
            cpu_usage_percent: cpu,
            cpu_usage_display: String::new(),
            memory_bytes: 0,
            memory_display: String::new(),
            memory_percent: 0.0,
            memory_percent_display: String::new(),
            virtual_memory_bytes: 0,
            virtual_memory_display: String::new(),
            disk_read_bytes_per_sec: 0,
            disk_read_display: String::new(),
            disk_written_bytes_per_sec: 0,
            disk_write_display: String::new(),
            disk_total_read_bytes: 0,
            disk_total_read_display: String::new(),
            disk_total_written_bytes: 0,
            disk_total_write_display: String::new(),
        }
    }
}
