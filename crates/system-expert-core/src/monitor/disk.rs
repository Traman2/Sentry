//! Storage volume capacity/usage (as opposed to per-process read/write throughput,
//! which lives on [`super::process::ProcessRow`]).

use serde::Serialize;
use sysinfo::Disks;

use super::units::{format_bytes, format_percent};

#[derive(Debug, Clone, Serialize)]
pub struct DiskMetrics {
    pub name: String,
    pub mount_point: String,
    pub file_system: String,
    pub kind: String,
    pub is_removable: bool,
    pub is_read_only: bool,

    pub total_bytes: u64,
    pub total_display: String,
    pub available_bytes: u64,
    pub available_display: String,
    pub used_bytes: u64,
    pub used_display: String,
    pub used_percent: f32,
    pub used_percent_display: String,
}

pub fn collect(disks: &Disks) -> Vec<DiskMetrics> {
    disks
        .list()
        .iter()
        .map(|disk| {
            let total_bytes = disk.total_space();
            let available_bytes = disk.available_space();
            let used_bytes = total_bytes.saturating_sub(available_bytes);
            let used_percent = if total_bytes > 0 {
                (used_bytes as f64 / total_bytes as f64 * 100.0) as f32
            } else {
                0.0
            };

            DiskMetrics {
                name: disk.name().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().to_string_lossy().into_owned(),
                file_system: disk.file_system().to_string_lossy().into_owned(),
                kind: disk.kind().to_string(),
                is_removable: disk.is_removable(),
                is_read_only: disk.is_read_only(),

                total_bytes,
                total_display: format_bytes(total_bytes),
                available_bytes,
                available_display: format_bytes(available_bytes),
                used_bytes,
                used_display: format_bytes(used_bytes),
                used_percent,
                used_percent_display: format_percent(used_percent),
            }
        })
        .collect()
}
