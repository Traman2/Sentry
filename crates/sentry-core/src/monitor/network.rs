//! System-wide network interface throughput.
//!
//! `sysinfo` only exposes network counters per network interface (not per process)
//! on any platform, so network activity is reported here rather than per process.

use serde::Serialize;
use sysinfo::Networks;

use super::units::{format_bytes, format_bytes_per_sec};

#[derive(Debug, Clone, Serialize)]
pub struct NetworkInterfaceMetrics {
    pub interface: String,
    pub mac_address: String,
    pub ip_addresses: Vec<String>,

    pub received_bytes_per_sec: u64,
    pub received_display: String,
    pub transmitted_bytes_per_sec: u64,
    pub transmitted_display: String,
    pub total_received_bytes: u64,
    pub total_received_display: String,
    pub total_transmitted_bytes: u64,
    pub total_transmitted_display: String,

    pub packets_received_per_sec: u64,
    pub packets_transmitted_per_sec: u64,
    pub errors_on_received: u64,
    pub errors_on_transmitted: u64,
}

pub fn collect(networks: &Networks) -> Vec<NetworkInterfaceMetrics> {
    networks
        .iter()
        .map(|(interface, data)| NetworkInterfaceMetrics {
            interface: interface.clone(),
            mac_address: data.mac_address().to_string(),
            ip_addresses: data.ip_networks().iter().map(|ip| ip.to_string()).collect(),

            received_bytes_per_sec: data.received(),
            received_display: format_bytes_per_sec(data.received()),
            transmitted_bytes_per_sec: data.transmitted(),
            transmitted_display: format_bytes_per_sec(data.transmitted()),
            total_received_bytes: data.total_received(),
            total_received_display: format_bytes(data.total_received()),
            total_transmitted_bytes: data.total_transmitted(),
            total_transmitted_display: format_bytes(data.total_transmitted()),

            packets_received_per_sec: data.packets_received(),
            packets_transmitted_per_sec: data.packets_transmitted(),
            errors_on_received: data.errors_on_received(),
            errors_on_transmitted: data.errors_on_transmitted(),
        })
        .collect()
}
