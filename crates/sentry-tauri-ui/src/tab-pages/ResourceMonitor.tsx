import { useState } from "react";
import {
  type SortingState,
  type VisibilityState,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { PaginationBar } from "./resource-monitor/PaginationBar";
import { ProcessTable } from "./resource-monitor/ProcessTable";
import { Toolbar } from "./resource-monitor/Toolbar";
import { processColumns } from "./resource-monitor/columns";
import { useSystemSnapshot } from "./resource-monitor/useSystemSnapshot";

function ResourceMonitor() {
  const snapshot = useSystemSnapshot();
  const [sorting, setSorting] = useState<SortingState>([
    { id: "cpu_usage_percent", desc: true },
  ]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});

  const table = useReactTable({
    data: snapshot.processes,
    columns: processColumns,
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
      <ProcessTable table={table} />
      <PaginationBar table={table} totalCount={snapshot.processes.length} />
    </div>
  );
}

export default ResourceMonitor;
