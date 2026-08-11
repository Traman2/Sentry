/**
 * Mirrors the serde-serialized shape of the Rust structs in
 * `crates/sentry-core/src/monitor/*.rs`. Kept in sync by hand — if you add or
 * rename a field over there, update it here too.
 */
export interface ProcessRow {
  pid: number;
  parent_pid: number | null;
  name: string;
  executable_path: string | null;
  command: string;
  status: string;
  user: string | null;
  effective_user: string | null;
  group_id: string | null;
  effective_group_id: string | null;
  session_id: number | null;
  thread_count: number | null;

  start_time_unix_secs: number;
  run_time_secs: number;

  cpu_usage_percent: number;
  cpu_usage_display: string;

  memory_bytes: number;
  memory_display: string;
  memory_percent: number;
  memory_percent_display: string;

  virtual_memory_bytes: number;
  virtual_memory_display: string;

  disk_read_bytes_per_sec: number;
  disk_read_display: string;
  disk_written_bytes_per_sec: number;
  disk_write_display: string;
  disk_total_read_bytes: number;
  disk_total_read_display: string;
  disk_total_written_bytes: number;
  disk_total_write_display: string;
}

/**
 * On-demand detail for a single process (environment variables, working
 * directory, root directory). Fetched separately via the `get_process_details`
 * command — deliberately not part of `ProcessRow`/`SystemSnapshot` since
 * environment variables can carry secrets and this is only useful for one
 * process at a time, not every process on every polling tick.
 */
export interface ProcessDetails {
  pid: number;
  current_working_directory: string | null;
  root_directory: string | null;
  environment: string[];
}

/** One row of the process table: an app aggregating one or more PIDs. */
export interface AppRow extends ProcessRow {
  pid_count: number;
  /** Individual PIDs backing this app, present only when pid_count > 1. */
  subRows?: AppRow[];
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

export interface DiskMetrics {
  name: string;
  mount_point: string;
  file_system: string;
  kind: string;
  is_removable: boolean;
  is_read_only: boolean;

  total_bytes: number;
  total_display: string;
  available_bytes: number;
  available_display: string;
  used_bytes: number;
  used_display: string;
  used_percent: number;
  used_percent_display: string;
}

export interface ComponentMetrics {
  label: string;
  temperature_celsius: number | null;
  max_temperature_celsius: number | null;
  critical_temperature_celsius: number | null;
}

export interface UserAccount {
  uid: string;
  gid: string;
  name: string;
  groups: string[];
}

export interface CoreUsage {
  name: string;
  vendor_id: string;
  brand: string;
  frequency_mhz: number;
  cpu_usage_percent: number;
  cpu_usage_display: string;
}

/** Always zero on Windows — `sysinfo` only implements load average on Unix-like platforms. */
export interface LoadAverage {
  one_minute: number;
  five_minutes: number;
  fifteen_minutes: number;
}

export interface SystemSummary {
  hostname: string | null;
  os_name: string | null;
  os_version: string | null;
  long_os_version: string | null;
  distribution_id: string;
  kernel_version: string | null;
  cpu_arch: string | null;

  uptime_secs: number;
  boot_time_unix_secs: number;
  load_average: LoadAverage;

  physical_core_count: number | null;
  logical_core_count: number;
  global_cpu_usage_percent: number;
  global_cpu_usage_display: string;
  per_core_usage: CoreUsage[];

  total_memory_bytes: number;
  total_memory_display: string;
  used_memory_bytes: number;
  used_memory_display: string;
  free_memory_bytes: number;
  free_memory_display: string;
  available_memory_bytes: number;
  available_memory_display: string;
  used_memory_percent: number;
  used_memory_percent_display: string;

  total_swap_bytes: number;
  total_swap_display: string;
  used_swap_bytes: number;
  used_swap_display: string;
  free_swap_bytes: number;
  free_swap_display: string;
}

export interface SystemSnapshot {
  timestamp_ms: number;
  system: SystemSummary;
  processes: ProcessRow[];
  networks: NetworkInterfaceMetrics[];
  disks: DiskMetrics[];
  components: ComponentMetrics[];
  users: UserAccount[];
}

/**
 * Mirrors `crates/sentry-core/src/history/query.rs`. One row recorded roughly every
 * 2 seconds by the background history recorder (`sentry_core::spawn_recorder`),
 * independent of whether the frontend is polling `get_snapshot`.
 */
export interface SystemSamplePoint {
  timestamp_ms: number;
  cpu_usage_percent: number;
  used_memory_bytes: number;
  total_memory_bytes: number;
  used_swap_bytes: number;
  network_rx_bytes_per_sec: number;
  network_tx_bytes_per_sec: number;
}

export interface ProcessSamplePoint {
  timestamp_ms: number;
  pid: number;
  name: string;
  cpu_usage_percent: number;
  memory_bytes: number;
  disk_read_bytes_per_sec: number;
  disk_written_bytes_per_sec: number;
}
