import type { ColumnDef } from "@tanstack/react-table";
import { ArrowDown, ArrowUp, ChevronRight } from "lucide-react";
import type { AppRow } from "./types";

function healthColor(percent: number) {
  if (percent >= 50) return "var(--color-danger)";
  if (percent >= 15) return "#c9a227";
  return "#3fa66b";
}

export const processColumns: ColumnDef<AppRow>[] = [
  {
    accessorKey: "name",
    header: "Name",
    size: 220,
    cell: ({ row }) => (
      <div
        className="flex items-center gap-3"
        style={{ paddingLeft: row.depth > 0 ? 28 : 0 }}
      >
        <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-teal/40 text-[10px] font-semibold text-navy">
          {row.original.name.slice(0, 1).toUpperCase()}
        </div>
        <div className="min-w-0">
          <div className="truncate text-xs font-medium text-navy">
            {row.original.name}
          </div>
          <div className="truncate text-[11px] text-muted-foreground">
            {row.depth === 0 && row.original.pid_count > 1
              ? `${row.original.pid_count} processes`
              : `PID ${row.original.pid}`}
          </div>
        </div>
        {row.getCanExpand() && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              row.toggleExpanded();
            }}
            className="ml-auto flex h-5 w-5 shrink-0 cursor-pointer items-center justify-center rounded hover:bg-teal/25"
          >
            <ChevronRight
              className={`h-3.5 w-3.5 text-navy/60 transition-transform ${row.getIsExpanded() ? "rotate-90" : ""}`}
            />
          </button>
        )}
      </div>
    ),
  },
  {
    accessorKey: "cpu_usage_percent",
    header: "CPU",
    size: 100,
    cell: ({ row }) => {
      const percent = Math.min(Math.max(row.original.cpu_usage_percent, 0), 100);
      const color = healthColor(row.original.cpu_usage_percent);
      const size = 12;
      const strokeWidth = 2;
      const radius = (size - strokeWidth) / 2;
      const circumference = 2 * Math.PI * radius;
      const dash = (percent / 100) * circumference;
      return (
        <div className="flex items-center gap-2">
          <svg
            width={size}
            height={size}
            viewBox={`0 0 ${size} ${size}`}
            className="-rotate-90 shrink-0"
          >
            <circle
              cx={size / 2}
              cy={size / 2}
              r={radius}
              fill="none"
              stroke="rgba(38,38,79,0.15)"
              strokeWidth={strokeWidth}
            />
            <circle
              cx={size / 2}
              cy={size / 2}
              r={radius}
              fill="none"
              stroke={color}
              strokeWidth={strokeWidth}
              strokeLinecap="round"
              strokeDasharray={`${dash} ${circumference}`}
            />
          </svg>
          <span className="text-xs text-navy">{row.original.cpu_usage_display}</span>
        </div>
      );
    },
  },
  {
    accessorKey: "memory_display",
    header: "Memory",
    size: 150,
    cell: ({ row }) => (
      <div>
        <div className="text-xs font-medium text-navy">
          {row.original.memory_display}
        </div>
        <div className="text-[11px] text-muted-foreground">
          {row.original.memory_percent_display} of system
        </div>
      </div>
    ),
  },
  {
    id: "storage",
    accessorFn: (row) => row.disk_read_bytes_per_sec + row.disk_written_bytes_per_sec,
    header: "Storage",
    size: 150,
    cell: ({ row }) => (
      <div className="flex flex-col gap-0.5 text-[11px] text-navy">
        <span className="flex items-center gap-1">
          <ArrowDown className="h-3 w-3 text-navy/40" />
          {row.original.disk_read_display}
        </span>
        <span className="flex items-center gap-1">
          <ArrowUp className="h-3 w-3 text-navy/40" />
          {row.original.disk_write_display}
        </span>
      </div>
    ),
  },
  {
    accessorKey: "executable_path",
    header: "Path",
    size: 340,
    cell: ({ row }) => (
      <span
        title={row.original.executable_path ?? undefined}
        className="block truncate text-[11px] text-muted-foreground"
      >
        {row.original.executable_path ?? "—"}
      </span>
    ),
  },
];
