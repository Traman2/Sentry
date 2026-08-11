//! Per-process metrics, shaped as flat table rows.

use serde::Serialize;
use sysinfo::{Process, System, Users};

use super::units::{format_bytes, format_bytes_per_sec, format_percent};

/// One row of a running-applications table, pre-formatted for direct rendering
/// (e.g. a TanStack Table column set).
///
/// Numeric fields are kept alongside `_display` counterparts so a UI can sort/filter on
/// the raw number while showing the formatted string.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessRow {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub executable_path: Option<String>,
    pub command: String,
    pub status: String,
    pub user: Option<String>,

    pub start_time_unix_secs: u64,
    pub run_time_secs: u64,

    pub cpu_usage_percent: f32,
    pub cpu_usage_display: String,

    pub memory_bytes: u64,
    pub memory_display: String,
    pub memory_percent: f32,
    pub memory_percent_display: String,

    pub virtual_memory_bytes: u64,
    pub virtual_memory_display: String,

    pub disk_read_bytes_per_sec: u64,
    pub disk_read_display: String,
    pub disk_written_bytes_per_sec: u64,
    pub disk_write_display: String,
    pub disk_total_read_bytes: u64,
    pub disk_total_read_display: String,
    pub disk_total_written_bytes: u64,
    pub disk_total_write_display: String,
}

/// Builds one [`ProcessRow`] per process known to `system`.
///
/// `total_memory_bytes` is used to compute each process's share of system memory.
/// `users` resolves each process's `user_id` to a username; an empty [`Users`] list
/// leaves the `user` field as `None` for every row.
///
/// Note: `sysinfo` only exposes network counters per network interface (not per
/// process) on any platform, so there is no per-process network column here — see
/// [`super::network`] for system-wide network throughput instead.
pub fn collect(system: &System, users: &Users, total_memory_bytes: u64) -> Vec<ProcessRow> {
    system
        .processes()
        .values()
        .map(|process| row_from_process(process, users, total_memory_bytes))
        .collect()
}

fn row_from_process(process: &Process, users: &Users, total_memory_bytes: u64) -> ProcessRow {
    let disk = process.disk_usage();
    let memory_bytes = process.memory();
    let memory_percent = if total_memory_bytes > 0 {
        (memory_bytes as f64 / total_memory_bytes as f64 * 100.0) as f32
    } else {
        0.0
    };

    ProcessRow {
        pid: process.pid().as_u32(),
        parent_pid: process.parent().map(|pid| pid.as_u32()),
        name: process.name().to_string_lossy().into_owned(),
        executable_path: process
            .exe()
            .map(|path| path.to_string_lossy().into_owned()),
        command: process
            .cmd()
            .iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" "),
        status: process.status().to_string(),
        user: process
            .user_id()
            .and_then(|uid| users.get_user_by_id(uid))
            .map(|user| user.name().to_string()),

        start_time_unix_secs: process.start_time(),
        run_time_secs: process.run_time(),

        cpu_usage_percent: process.cpu_usage(),
        cpu_usage_display: format_percent(process.cpu_usage()),

        memory_bytes,
        memory_display: format_bytes(memory_bytes),
        memory_percent,
        memory_percent_display: format_percent(memory_percent),

        virtual_memory_bytes: process.virtual_memory(),
        virtual_memory_display: format_bytes(process.virtual_memory()),

        disk_read_bytes_per_sec: disk.read_bytes,
        disk_read_display: format_bytes_per_sec(disk.read_bytes),
        disk_written_bytes_per_sec: disk.written_bytes,
        disk_write_display: format_bytes_per_sec(disk.written_bytes),
        disk_total_read_bytes: disk.total_read_bytes,
        disk_total_read_display: format_bytes(disk.total_read_bytes),
        disk_total_written_bytes: disk.total_written_bytes,
        disk_total_write_display: format_bytes(disk.total_written_bytes),
    }
}
