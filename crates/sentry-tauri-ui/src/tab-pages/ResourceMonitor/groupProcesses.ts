import { formatBytes, formatPercent, formatRate } from "./format";
import type { AppRow, ProcessRow } from "./types";

/**
 * Groups per-pid processes into one row per app name, summing usage across
 * every pid that shares a name. Apps with more than one pid keep their raw
 * processes as `subRows` so the table can expand into a per-pid breakdown.
 */
export function groupProcessesByApp(processes: ProcessRow[]): AppRow[] {
  const byName = new Map<string, ProcessRow[]>();
  for (const process of processes) {
    const group = byName.get(process.name);
    if (group) {
      group.push(process);
    } else {
      byName.set(process.name, [process]);
    }
  }

  return Array.from(byName.values()).map((group): AppRow => {
    if (group.length === 1) {
      return { ...group[0], pid_count: 1 };
    }

    const cpu_usage_percent = sum(group, (p) => p.cpu_usage_percent);
    const memory_bytes = sum(group, (p) => p.memory_bytes);
    const memory_percent = sum(group, (p) => p.memory_percent);
    const disk_read_bytes_per_sec = sum(group, (p) => p.disk_read_bytes_per_sec);
    const disk_written_bytes_per_sec = sum(group, (p) => p.disk_written_bytes_per_sec);

    const representative = group.reduce((a, b) => (a.pid < b.pid ? a : b));

    const aggregate: AppRow = {
      ...representative,
      cpu_usage_percent,
      cpu_usage_display: formatPercent(cpu_usage_percent),
      memory_bytes,
      memory_display: formatBytes(memory_bytes),
      memory_percent,
      memory_percent_display: formatPercent(memory_percent),
      disk_read_bytes_per_sec,
      disk_read_display: formatRate(disk_read_bytes_per_sec),
      disk_written_bytes_per_sec,
      disk_write_display: formatRate(disk_written_bytes_per_sec),
      pid_count: group.length,
      subRows: group
        .slice()
        .sort((a, b) => a.pid - b.pid)
        .map((process): AppRow => ({ ...process, pid_count: 1 })),
    };
    return aggregate;
  });
}

function sum<T>(items: T[], select: (item: T) => number): number {
  return items.reduce((total, item) => total + select(item), 0);
}

/** Finds a row (top-level app or one of its subRows) by pid. */
export function findAppRowByPid(rows: AppRow[], pid: number): AppRow | null {
  for (const row of rows) {
    if (row.pid === pid) return row;
    const child = row.subRows?.find((subRow) => subRow.pid === pid);
    if (child) return child;
  }
  return null;
}
