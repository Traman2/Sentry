import { invoke } from "@tauri-apps/api/core";
import {
  type ColumnDef,
  type SortingState,
  type Table,
  type VisibilityState,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";

import {
  ArrowUpDown,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  Columns3,
  SlidersHorizontal,
  Upload,
} from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "../components/ui/dropdown-menu";

interface ProcessRow {
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
}

function healthColor(percent: number) {
  if (percent >= 50) return "bg-danger";
  if (percent >= 15) return "#c9a227";
  return "#3fa66b";
}

function ToolbarButton({
  icon,
  label,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  onClick?: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="flex cursor-pointer items-center gap-1.5 rounded-md px-2 py-1 text-sm text-navy/70 hover:bg-teal/25"
    >
      {icon}
      {label}
    </button>
  );
}

function Toolbar({ table }: { table: Table<ProcessRow> }) {
  const nameColumn = table.getColumn("name");
  return (
    <div className="flex items-center justify-between border-b border-teal px-4 py-2.5">
      <button className="flex cursor-pointer items-center gap-1.5 rounded-md px-2 py-1 text-sm font-semibold text-navy hover:bg-teal/25">
        All Processes
        <ChevronDown className="h-3.5 w-3.5" />
      </button>
      <div className="flex items-center gap-1">
        <DropdownMenu>
          <DropdownMenuTrigger className="flex cursor-pointer items-center gap-1.5 rounded-md px-2 py-1 text-sm text-navy/70 hover:bg-teal/25">
            <Columns3 className="h-3.5 w-3.5" />
            Columns
          </DropdownMenuTrigger>
          <DropdownMenuContent>
            <DropdownMenuGroup>
              <DropdownMenuLabel>Toggle columns</DropdownMenuLabel>
              <DropdownMenuSeparator />
              {table.getAllLeafColumns().map((column) => (
                <DropdownMenuCheckboxItem
                  key={column.id}
                  checked={column.getIsVisible()}
                  onCheckedChange={(checked) => column.toggleVisibility(!!checked)}
                >
                  {String(column.columnDef.header)}
                </DropdownMenuCheckboxItem>
              ))}
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>

        <DropdownMenu>
          <DropdownMenuTrigger className="flex cursor-pointer items-center gap-1.5 rounded-md px-2 py-1 text-sm text-navy/70 hover:bg-teal/25">
            <SlidersHorizontal className="h-3.5 w-3.5" />
            Filters
          </DropdownMenuTrigger>
          <DropdownMenuContent className="w-56">
            <DropdownMenuGroup>
              <DropdownMenuLabel>Filter by name</DropdownMenuLabel>
              <DropdownMenuSeparator />
              <div className="px-1.5 py-1">
                <input
                  autoFocus
                  value={(nameColumn?.getFilterValue() as string) ?? ""}
                  onChange={(e) => nameColumn?.setFilterValue(e.target.value)}
                  onKeyDown={(e) => e.stopPropagation()}
                  placeholder="Process name…"
                  className="w-full rounded-md border border-teal bg-canvas px-2 py-1 text-sm text-navy outline-none focus:border-navy/40"
                />
              </div>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>

        <ToolbarButton icon={<Upload className="h-3.5 w-3.5" />} label="Export" />
      </div>
    </div>
  );
}

function ResourceMonitor() {
  const [processes, setProcesses] = useState<ProcessRow[]>([]);
  const [sorting, setSorting] = useState<SortingState>([
    { id: "cpu_usage_percent", desc: true },
  ]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});

  useEffect(() => {
    let cancelled = false;

    async function refresh() {
      const rows = await invoke<ProcessRow[]>("get_processes");
      if (!cancelled) setProcesses(rows);
    }

    refresh();
    const interval = setInterval(refresh, 2000);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, []);

  const columns = useMemo<ColumnDef<ProcessRow>[]>(
    () => [
      {
        accessorKey: "name",
        header: "Name",
        size: 220,
        cell: ({ row }) => (
          <div className="flex items-center gap-3">
            <div className="flex h-6 w-6 items-center justify-center rounded-full bg-teal/40 text-[10px] font-semibold text-navy">
              {row.original.name.slice(0, 1).toUpperCase()}
            </div>
            <div className="min-w-0">
              <div className="truncate text-xs font-medium text-navy">
                {row.original.name}
              </div>
              <div className="truncate text-[11px] text-muted-foreground">
                PID {row.original.pid}
              </div>
            </div>
          </div>
        ),
      },
      {
        accessorKey: "cpu_usage_percent",
        header: "CPU",
        size: 100,
        cell: ({ row }) => (
          <div className="flex items-center gap-2">
            <span
              className="h-1.5 w-1.5 rounded-full"
              style={{ background: healthColor(row.original.cpu_usage_percent) }}
            />
            <span className="text-xs text-navy">{row.original.cpu_usage_display}</span>
          </div>
        ),
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
        accessorKey: "status",
        header: "Status",
        size: 130,
        cell: ({ row }) => (
          <span className="inline-flex items-center gap-1.5 rounded-full border border-teal px-2 py-1 text-[11px] text-navy">
            <span className="h-1.5 w-1.5 rounded-full bg-teal" />
            {row.original.status}
          </span>
        ),
      },
      {
        accessorKey: "executable_path",
        header: "Path",
        size: 280,
        cell: ({ row }) => (
          <span
            title={row.original.executable_path ?? undefined}
            className="block truncate text-[11px] text-muted-foreground"
          >
            {row.original.executable_path ?? "—"}
          </span>
        ),
      },
    ],
    [],
  );

  const table = useReactTable({
    data: processes,
    columns,
    state: { sorting, columnVisibility },
    onSortingChange: setSorting,
    onColumnVisibilityChange: setColumnVisibility,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    initialState: { pagination: { pageSize: 15 } },
    autoResetPageIndex: false,
  });

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm">
      <Toolbar table={table} />
      <div className="flex-1 overflow-y-auto overflow-x-auto">
        <table className="w-full text-left" style={{ tableLayout: "fixed", borderCollapse: "collapse" }}>
          <colgroup>
            {table.getFlatHeaders().map((header) => (
              <col key={header.id} style={{ width: header.getSize() }} />
            ))}
          </colgroup>
          <thead className="sticky top-0 bg-canvas">
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id} className="border-b border-teal">
                {headerGroup.headers.map((header) => (
                  <th
                    key={header.id}
                    className="border-r border-teal/50 px-4 py-1.5 text-[10px] font-semibold uppercase tracking-wide text-navy/60 last:border-r-0"
                  >
                    {header.isPlaceholder ? null : (
                      <button
                        onClick={header.column.getToggleSortingHandler()}
                        className="flex cursor-pointer items-center gap-1 hover:text-navy"
                      >
                        {flexRender(header.column.columnDef.header, header.getContext())}
                        {header.column.getCanSort() && (
                          <ArrowUpDown className="h-3 w-3" />
                        )}
                      </button>
                    )}
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows.map((row) => (
              <tr
                key={row.id}
                className="border-b border-teal/50 hover:bg-teal/10"
              >
                {row.getVisibleCells().map((cell) => (
                  <td
                    key={cell.id}
                    className="overflow-hidden border-r border-teal/50 px-4 py-1.5 last:border-r-0"
                  >
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="flex items-center justify-between border-t border-teal px-4 py-2.5 text-[11px] text-muted-foreground">
        <span>
          Page {table.getState().pagination.pageIndex + 1} of{" "}
          {Math.max(table.getPageCount(), 1)} · {processes.length} processes
        </span>
        <div className="flex items-center gap-1">
          <button
            onClick={() => table.previousPage()}
            disabled={!table.getCanPreviousPage()}
            className="flex h-6 w-6 cursor-pointer items-center justify-center rounded disabled:cursor-not-allowed disabled:opacity-40 hover:bg-teal/25"
          >
            <ChevronLeft className="h-3.5 w-3.5" />
          </button>
          <button
            onClick={() => table.nextPage()}
            disabled={!table.getCanNextPage()}
            className="flex h-6 w-6 cursor-pointer items-center justify-center rounded disabled:cursor-not-allowed disabled:opacity-40 hover:bg-teal/25"
          >
            <ChevronRight className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>
    </div>
  );
}

export default ResourceMonitor;
