import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { SystemSnapshot, SystemSummary } from "./types";

const EMPTY_SYSTEM_SUMMARY: SystemSummary = {
  hostname: null,
  os_name: null,
  os_version: null,
  long_os_version: null,
  distribution_id: "",
  kernel_version: null,
  cpu_arch: null,

  uptime_secs: 0,
  boot_time_unix_secs: 0,
  load_average: { one_minute: 0, five_minutes: 0, fifteen_minutes: 0 },

  physical_core_count: null,
  logical_core_count: 0,
  global_cpu_usage_percent: 0,
  global_cpu_usage_display: "0.0%",
  per_core_usage: [],

  total_memory_bytes: 0,
  total_memory_display: "0 B",
  used_memory_bytes: 0,
  used_memory_display: "0 B",
  free_memory_bytes: 0,
  free_memory_display: "0 B",
  available_memory_bytes: 0,
  available_memory_display: "0 B",
  used_memory_percent: 0,
  used_memory_percent_display: "0.0%",

  total_swap_bytes: 0,
  total_swap_display: "0 B",
  used_swap_bytes: 0,
  used_swap_display: "0 B",
  free_swap_bytes: 0,
  free_swap_display: "0 B",
};

const EMPTY_SNAPSHOT: SystemSnapshot = {
  timestamp_ms: 0,
  system: EMPTY_SYSTEM_SUMMARY,
  processes: [],
  networks: [],
  disks: [],
  components: [],
  users: [],
};

export function useSystemSnapshot(intervalMs = 2000) {
  const [snapshot, setSnapshot] = useState<SystemSnapshot>(EMPTY_SNAPSHOT);

  useEffect(() => {
    let cancelled = false;

    async function refresh() {
      const next = await invoke<SystemSnapshot>("get_snapshot");
      if (!cancelled) setSnapshot(next);
    }

    refresh();
    const interval = setInterval(refresh, intervalMs);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [intervalMs]);

  return snapshot;
}
