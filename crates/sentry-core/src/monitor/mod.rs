//! Real-time system/application activity monitoring, backed by `sysinfo`.
//!
//! Data is organized into focused submodules, each producing table-ready rows
//! (raw numeric fields alongside pre-formatted `_display` strings) suitable for
//! rendering directly in a UI table:
//!
//! - [`process`] — one row per running process/application (CPU, memory, disk I/O).
//! - [`network`] — one row per network interface (system-wide throughput; `sysinfo`
//!   cannot attribute network traffic to individual processes on any platform).
//! - [`disk`] — one row per storage volume (capacity/used/free).
//! - [`system`] — a single whole-machine summary (OS identity, uptime, aggregate
//!   CPU/memory/swap).
//! - [`components`] — one row per hardware temperature sensor, where available.
//! - [`users`] — one row per system user account.
//! - [`units`] — shared byte/percentage formatting helpers.

pub mod components;
pub mod disk;
pub mod network;
pub mod process;
mod stream;
pub mod system;
pub mod units;
pub mod users;

pub use components::ComponentMetrics;
pub use disk::DiskMetrics;
pub use network::NetworkInterfaceMetrics;
pub use process::{ProcessDetails, ProcessRow};
pub use stream::{spawn_stream, stream_json};
pub use system::SystemSummary;
pub use users::UserAccount;

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sysinfo::{
    Components, Disks, Networks, Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind,
    Users,
};

/// A single point-in-time capture of the whole machine: system summary, every
/// process, every network interface, every storage volume, every temperature
/// sensor, and every user account.
#[derive(Debug, Clone, Serialize)]
pub struct SystemSnapshot {
    pub timestamp_ms: u128,
    pub system: SystemSummary,
    pub processes: Vec<ProcessRow>,
    pub networks: Vec<NetworkInterfaceMetrics>,
    pub disks: Vec<DiskMetrics>,
    pub components: Vec<ComponentMetrics>,
    pub users: Vec<UserAccount>,
}

/// Holds the `sysinfo` handles that need to persist across refreshes so that
/// per-second deltas (disk I/O, network throughput, CPU usage) are correct.
pub struct Monitor {
    system: System,
    networks: Networks,
    disks: Disks,
    users: Users,
    components: Components,
}

impl Monitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let users = Users::new_with_refreshed_list();
        let components = Components::new_with_refreshed_list();
        Self {
            system,
            networks,
            disks,
            users,
            components,
        }
    }

    pub fn snapshot(&mut self) -> SystemSnapshot {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.networks.refresh();
        self.disks.refresh();
        self.components.refresh();

        SystemSnapshot {
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            system: system::collect(&self.system),
            processes: process::collect(&self.system, &self.users, self.system.total_memory()),
            networks: network::collect(&self.networks),
            disks: disk::collect(&self.disks),
            components: components::collect(&self.components),
            users: users::collect(&self.users),
        }
    }

    pub fn snapshot_json(&mut self) -> serde_json::Result<String> {
        serde_json::to_string(&self.snapshot())
    }

    /// Fetches [`process::ProcessDetails`] (environment, cwd, root) for a single `pid`,
    /// refreshing just that process. Kept separate from [`Monitor::snapshot`] so this
    /// data — which can include secrets carried in environment variables — is only
    /// read when a caller explicitly asks about one specific process, not broadcast
    /// for every process on every polling tick.
    pub fn process_details(&mut self, pid: u32) -> Option<process::ProcessDetails> {
        let sysinfo_pid = Pid::from_u32(pid);
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[sysinfo_pid]),
            false,
            ProcessRefreshKind::new()
                .with_cwd(UpdateKind::Always)
                .with_root(UpdateKind::Always)
                .with_environ(UpdateKind::Always),
        );
        process::details(&self.system, pid)
    }
}

impl Default for Monitor {
    fn default() -> Self {
        Self::new()
    }
}
