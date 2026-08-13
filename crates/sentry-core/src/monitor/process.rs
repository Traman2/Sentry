//! Per-process metrics, shaped as flat table rows.

use serde::Serialize;
use sysinfo::{Pid, Process, Signal, System, Users};

use super::units::{format_bytes, format_bytes_per_sec, format_percent};

/// One row of a running-applications table, pre-formatted for direct rendering
/// (e.g. a TanStack Table column set).
///
/// Numeric fields are kept alongside `_display` counterparts so a UI can sort/filter on
/// the raw number while showing the formatted string.
///
/// Deliberately excludes a process's environment variables, working directory, and
/// root directory: those can contain secrets (tokens, credentials in env vars) and
/// are only useful when inspecting one specific process, not for a table listing
/// every process on the machine every polling tick. Fetch them on demand instead via
/// [`details`].
#[derive(Debug, Clone, Serialize)]
pub struct ProcessRow {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub name: String,
    pub executable_path: Option<String>,
    pub command: String,
    pub status: String,
    pub user: Option<String>,
    pub effective_user: Option<String>,
    pub group_id: Option<String>,
    pub effective_group_id: Option<String>,
    pub session_id: Option<u32>,
    pub thread_count: Option<usize>,

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
        effective_user: process
            .effective_user_id()
            .and_then(|uid| users.get_user_by_id(uid))
            .map(|user| user.name().to_string()),
        group_id: process.group_id().map(|gid| gid.to_string()),
        effective_group_id: process.effective_group_id().map(|gid| gid.to_string()),
        session_id: process.session_id().map(|pid| pid.as_u32()),
        thread_count: process.tasks().map(|tasks| tasks.len()),

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

/// On-demand detail for a single process: environment variables, working directory,
/// and root directory. Kept out of [`ProcessRow`] (and so out of every polling-interval
/// snapshot) because environment variables can carry secrets and this data is only
/// useful when a user has drilled into one specific process.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessDetails {
    pub pid: u32,
    /// The process's name, carried here so a caller that looked up one pid can confirm what
    /// it actually inspected without a second, full-table refresh just to resolve a name.
    pub name: String,
    pub current_working_directory: Option<String>,
    pub root_directory: Option<String>,
    pub environment: Vec<String>,
}

/// The result of attempting to terminate a process.
///
/// Every field is populated on both success and failure so a caller can tell the three
/// outcomes apart without inspecting an error string: the process wasn't there
/// (`found: false`), it was there but the signal couldn't be delivered
/// (`found: true, delivered: false`), or it was signalled (`delivered: true`).
#[derive(Debug, Clone, Serialize)]
pub struct KillOutcome {
    pub pid: u32,
    /// The name of the process as it was at the moment of the attempt, when one was found.
    pub name: Option<String>,
    /// Whether a process with this pid existed in `system`'s process list.
    pub found: bool,
    /// The signal that was attempted, e.g. `"Kill"`.
    pub signal: String,
    /// Whether the signal was actually delivered.
    pub delivered: bool,
    /// Human-readable explanation, suitable for surfacing to a user or an agent.
    pub message: String,
}

/// Sends `signal` (default [`Signal::Kill`]) to `pid`.
///
/// Operates on `system`'s **last-refreshed** process list — it does not refresh. Callers
/// that might be holding a stale table should go through [`super::Monitor::kill_process`],
/// which refreshes the single pid first; killing from a stale table risks terminating an
/// unrelated process that has since inherited a recycled pid.
///
/// `sysinfo` reports an unsupported signal (`kill_with` returning `None`) distinctly from a
/// failed delivery; both come back as `delivered: false`, distinguished by `message`.
pub fn kill(system: &System, pid: u32, signal: Option<Signal>) -> KillOutcome {
    let signal = signal.unwrap_or(Signal::Kill);
    let Some(process) = system.process(Pid::from_u32(pid)) else {
        return KillOutcome {
            pid,
            name: None,
            found: false,
            signal: format!("{signal:?}"),
            delivered: false,
            message: format!("no process with pid {pid}"),
        };
    };

    let name = process.name().to_string_lossy().into_owned();
    let (delivered, message) = match process.kill_with(signal) {
        Some(true) => (true, format!("sent {signal:?} to {name} (pid {pid})")),
        Some(false) => (
            false,
            format!("the OS refused to deliver {signal:?} to {name} (pid {pid}) — it may be protected or already exiting"),
        ),
        None => (
            false,
            format!("{signal:?} is not supported on this platform"),
        ),
    };

    KillOutcome {
        pid,
        name: Some(name),
        found: true,
        signal: format!("{signal:?}"),
        delivered,
        message,
    }
}

/// Looks up [`ProcessDetails`] for a single `pid`, or `None` if the process no longer
/// exists in `system`'s last-refreshed process list.
pub fn details(system: &System, pid: u32) -> Option<ProcessDetails> {
    let process = system.process(Pid::from_u32(pid))?;
    Some(ProcessDetails {
        pid,
        name: process.name().to_string_lossy().into_owned(),
        current_working_directory: process
            .cwd()
            .map(|path| path.to_string_lossy().into_owned()),
        root_directory: process
            .root()
            .map(|path| path.to_string_lossy().into_owned()),
        environment: process
            .environ()
            .iter()
            .map(|var| var.to_string_lossy().into_owned())
            .collect(),
    })
}
