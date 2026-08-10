export interface ProcessRow {
  pid: number;
  parent_pid: number | null;
  name: string;
  executable_path: string | null;
  status: string;
  start_time_unix_secs: number;
  cpu_usage_percent: number;
  cpu_usage_display: string;
  memory_display: string;
  memory_percent_display: string;

  disk_read_bytes_per_sec: number;
  disk_read_display: string;
  disk_written_bytes_per_sec: number;
  disk_write_display: string;
}

export interface NetworkInterfaceMetrics {
  interface: string;
  mac_address: string;
  ip_addresses: string[];

  received_bytes_per_sec: number;
  received_display: string;
  transmitted_bytes_per_sec: number;
  transmitted_display: string;
  total_received_bytes: number;
  total_received_display: string;
  total_transmitted_bytes: number;
  total_transmitted_display: string;

  packets_received_per_sec: number;
  packets_transmitted_per_sec: number;
  errors_on_received: number;
  errors_on_transmitted: number;
}

export interface SystemSnapshot {
  timestamp_ms: number;
  processes: ProcessRow[];
  networks: NetworkInterfaceMetrics[];
}