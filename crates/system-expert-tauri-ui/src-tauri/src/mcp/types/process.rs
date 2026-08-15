//! Process rows projected and paged down for a tool result. See the module doc on
//! [`super`] for why the projection exists at all.

use serde::Serialize;
use system_expert_core::ProcessRow;

/// One process, reduced to what an agent reasons over. Compare
/// [`system_expert_core::ProcessRow`]'s 30 fields.
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
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProcessSort {
    /// Highest CPU first — the default, and what "what's slowing my machine down" means.
    #[default]
    Cpu,
    /// Highest resident memory first.
    Memory,
    /// Highest combined disk read+write rate first.
    Disk,
    /// Alphabetical by process name.
    Name,
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
        ProcessSort::Memory => matched.sort_by_key(|r| std::cmp::Reverse(r.memory_bytes)),
        ProcessSort::Disk => matched.sort_by_key(|r| {
            std::cmp::Reverse(r.disk_read_bytes_per_sec + r.disk_written_bytes_per_sec)
        }),
        ProcessSort::Name => matched.sort_by_key(|r| r.name.to_lowercase()),
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

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
