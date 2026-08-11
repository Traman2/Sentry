import type { ColumnDef } from "@tanstack/react-table";
import { ArrowDown, ArrowUp } from "lucide-react";
import Gauge from "@/components/Gauge";
import { formatBytes, formatPercent, formatRate } from "../ResourceMonitor/format";
import type { ProcessSamplePoint } from "../ResourceMonitor/types";
import { formatTick } from "./format";

const timeColumn: ColumnDef<ProcessSamplePoint> = {
  accessorKey: "timestamp_ms",
  header: "Time",
  size: 100,
  cell: ({ row }) => (
    <span className="text-xs text-navy tabular-nums">{formatTick(row.original.timestamp_ms)}</span>
  ),
};

function NameCell({ row }: { row: ProcessSamplePoint }) {
  return (
    <div className="flex items-center gap-3">
      <div className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-teal/40 text-[10px] font-semibold text-navy">
        {row.name.slice(0, 1).toUpperCase()}
      </div>
      <div className="min-w-0">
        <div className="truncate text-xs font-medium text-navy">{row.name}</div>
        <div className="truncate text-[11px] text-muted-foreground">PID {row.pid}</div>
      </div>
    </div>
  );
}

export const cpuColumns: ColumnDef<ProcessSamplePoint>[] = [
  timeColumn,
  {
    accessorKey: "name",
    header: "Name",
    size: 220,
    cell: ({ row }) => <NameCell row={row.original} />,
  },
  {
    accessorKey: "cpu_usage_percent",
    header: "CPU",
    size: 150,
    cell: ({ row }) => (
      <div className="flex items-center gap-2">
        <Gauge percent={row.original.cpu_usage_percent} />
        <span className="text-xs text-navy">{formatPercent(row.original.cpu_usage_percent)}</span>
      </div>
    ),
  },
];

export const memoryColumns: ColumnDef<ProcessSamplePoint>[] = [
  timeColumn,
  {
    accessorKey: "name",
    header: "Name",
    size: 220,
    cell: ({ row }) => <NameCell row={row.original} />,
  },
  {
    accessorKey: "memory_bytes",
    header: "Memory",
    size: 150,
    cell: ({ row }) => (
      <span className="text-xs font-medium text-navy">
        {formatBytes(row.original.memory_bytes)}
      </span>
    ),
  },
];

export const storageColumns: ColumnDef<ProcessSamplePoint>[] = [
  timeColumn,
  {
    accessorKey: "name",
    header: "Name",
    size: 220,
    cell: ({ row }) => <NameCell row={row.original} />,
  },
  {
    id: "storage",
    accessorFn: (row) => row.disk_read_bytes_per_sec + row.disk_written_bytes_per_sec,
    header: "Storage",
    size: 180,
    cell: ({ row }) => (
      <div className="flex flex-col gap-0.5 text-[11px] text-navy">
        <span className="flex items-center gap-1">
          <ArrowDown className="h-3 w-3 text-navy/40" />
          {formatRate(row.original.disk_read_bytes_per_sec)}
        </span>
        <span className="flex items-center gap-1">
          <ArrowUp className="h-3 w-3 text-navy/40" />
          {formatRate(row.original.disk_written_bytes_per_sec)}
        </span>
      </div>
    ),
  },
];
