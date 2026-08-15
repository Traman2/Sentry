export type MetricView = "cpu" | "memory" | "storage";

export type RangeKey = "1m" | "10m" | "24h";

export const RANGE_OPTIONS: Record<RangeKey, { label: string; secs: number; refetchMs: number }> = {
  "1m": { label: "1 min", secs: 60, refetchMs: 2000 },
  "10m": { label: "10 min", secs: 600, refetchMs: 3000 },
  "24h": { label: "24 hr", secs: 24 * 60 * 60, refetchMs: 10_000 },
};

export const METRIC_LABELS: Record<MetricView, string> = {
  cpu: "CPU",
  memory: "Memory",
  storage: "Storage",
};

export const CPU_COLOR = "#26264f";
export const MEMORY_COLOR = "#5a9184";
export const DISK_READ_COLOR = "#0ea5e9";
export const DISK_WRITE_COLOR = "#f59e0b";

// The number of consecutive liveness polls (2s apart) that must all miss every
// tracked pid before we treat the process as gone. One miss alone could just be
// a slow snapshot tick landing between samples.
export const DEATH_CONFIRMATION_POLLS = 2;
