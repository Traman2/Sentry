//! Whole-machine summary: OS identity, uptime, aggregate CPU and memory.

use serde::Serialize;
use sysinfo::System;

use super::units::{format_bytes, format_percent};

#[derive(Debug, Clone, Serialize)]
pub struct CoreUsage {
    pub name: String,
    pub cpu_usage_percent: f32,
    pub cpu_usage_display: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemSummary {
    pub hostname: Option<String>,
    pub os_name: Option<String>,
    pub os_version: Option<String>,
    pub kernel_version: Option<String>,

    pub uptime_secs: u64,
    pub boot_time_unix_secs: u64,

    pub physical_core_count: Option<usize>,
    pub logical_core_count: usize,
    pub global_cpu_usage_percent: f32,
    pub global_cpu_usage_display: String,
    pub per_core_usage: Vec<CoreUsage>,

    pub total_memory_bytes: u64,
    pub total_memory_display: String,
    pub used_memory_bytes: u64,
    pub used_memory_display: String,
    pub available_memory_bytes: u64,
    pub available_memory_display: String,
    pub used_memory_percent: f32,
    pub used_memory_percent_display: String,

    pub total_swap_bytes: u64,
    pub total_swap_display: String,
    pub used_swap_bytes: u64,
    pub used_swap_display: String,
}

pub fn collect(system: &System) -> SystemSummary {
    let total_memory_bytes = system.total_memory();
    let used_memory_bytes = system.used_memory();
    let used_memory_percent = if total_memory_bytes > 0 {
        (used_memory_bytes as f64 / total_memory_bytes as f64 * 100.0) as f32
    } else {
        0.0
    };

    let per_core_usage = system
        .cpus()
        .iter()
        .map(|cpu| CoreUsage {
            name: cpu.name().to_string(),
            cpu_usage_percent: cpu.cpu_usage(),
            cpu_usage_display: format_percent(cpu.cpu_usage()),
        })
        .collect();

    SystemSummary {
        hostname: System::host_name(),
        os_name: System::name(),
        os_version: System::os_version(),
        kernel_version: System::kernel_version(),

        uptime_secs: System::uptime(),
        boot_time_unix_secs: System::boot_time(),

        physical_core_count: system.physical_core_count(),
        logical_core_count: system.cpus().len(),
        global_cpu_usage_percent: system.global_cpu_usage(),
        global_cpu_usage_display: format_percent(system.global_cpu_usage()),
        per_core_usage,

        total_memory_bytes,
        total_memory_display: format_bytes(total_memory_bytes),
        used_memory_bytes,
        used_memory_display: format_bytes(used_memory_bytes),
        available_memory_bytes: system.available_memory(),
        available_memory_display: format_bytes(system.available_memory()),
        used_memory_percent,
        used_memory_percent_display: format_percent(used_memory_percent),

        total_swap_bytes: system.total_swap(),
        total_swap_display: format_bytes(system.total_swap()),
        used_swap_bytes: system.used_swap(),
        used_swap_display: format_bytes(system.used_swap()),
    }
}
