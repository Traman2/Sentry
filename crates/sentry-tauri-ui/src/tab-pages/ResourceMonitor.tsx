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
import { SelectionActionBar } from "./resource-monitor/SelectionActionBar";
import { Toolbar } from "./resource-monitor/Toolbar";
import { processColumns } from "./resource-monitor/columns";
import type { ProcessRow } from "./resource-monitor/types";
import { useSystemSnapshot } from "./resource-monitor/useSystemSnapshot";

function ResourceMonitor() {
  const snapshot = useSystemSnapshot();
  const [sorting, setSorting] = useState<SortingState>([
    { id: "cpu_usage_percent", desc: true },
  ]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [selectedPid, setSelectedPid] = useState<number | null>(null);

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
    initialState: { pagination: { pageSize: 40 } },
    autoResetPageIndex: false,
  });

  const selectedProcess = snapshot.processes.find((p) => p.pid === selectedPid) ?? null;

  const handleSelectRow = (pid: number) => {
    setSelectedPid((current) => (current === pid ? null : pid));
  };

  const handleViewMore = (process: ProcessRow) => {
    console.log("View more", process);
  };

  const handleTrack = (process: ProcessRow) => {
    console.log("Track", process);
  };

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm">
      <Toolbar table={table} />
      <ProcessTable table={table} selectedPid={selectedPid} onSelectRow={handleSelectRow} />
      {selectedProcess && (
        <SelectionActionBar
          process={selectedProcess}
          onViewMore={handleViewMore}
          onTrack={handleTrack}
        />
      )}
      <PaginationBar table={table} totalCount={snapshot.processes.length} />
    </div>
  );
}

export default ResourceMonitor;
