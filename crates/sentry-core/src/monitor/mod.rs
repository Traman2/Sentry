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
//! - [`units`] — shared byte/percentage formatting helpers.

pub mod disk;
pub mod network;
pub mod process;
mod stream;
pub mod system;
pub mod units;

pub use disk::DiskMetrics;
pub use network::NetworkInterfaceMetrics;
pub use process::ProcessRow;
pub use stream::{spawn_stream, stream_json};
pub use system::SystemSummary;

use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use sysinfo::{Disks, Networks, ProcessesToUpdate, System, Users};

/// A single point-in-time capture of the whole machine: system summary, every
/// process, every network interface, and every storage volume.
#[derive(Debug, Clone, Serialize)]
pub struct SystemSnapshot {
    pub timestamp_ms: u128,
    pub system: SystemSummary,
    pub processes: Vec<ProcessRow>,
    pub networks: Vec<NetworkInterfaceMetrics>,
    pub disks: Vec<DiskMetrics>,
}

/// Holds the `sysinfo` handles that need to persist across refreshes so that
/// per-second deltas (disk I/O, network throughput, CPU usage) are correct.
pub struct Monitor {
    system: System,
    networks: Networks,
    disks: Disks,
    users: Users,
}

impl Monitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let users = Users::new_with_refreshed_list();
        Self {
            system,
            networks,
            disks,
            users,
        }
    }

    pub fn snapshot(&mut self) -> SystemSnapshot {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.system.refresh_processes(ProcessesToUpdate::All, true);
        self.networks.refresh();
        self.disks.refresh();

        SystemSnapshot {
            timestamp_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            system: system::collect(&self.system),
            processes: process::collect(&self.system, &self.users, self.system.total_memory()),
            networks: network::collect(&self.networks),
            disks: disk::collect(&self.disks),
        }
    }

    pub fn snapshot_json(&mut self) -> serde_json::Result<String> {
        serde_json::to_string(&self.snapshot())
    }
}

impl Default for Monitor {
    fn default() -> Self {
        Self::new()
    }
}
