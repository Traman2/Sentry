import { useMemo, useState } from "react";
import {
  type ExpandedState,
  type SortingState,
  type VisibilityState,
  getCoreRowModel,
  getExpandedRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { PaginationBar } from "./ResourceMonitor/PaginationBar";
import { ProcessTable } from "./ResourceMonitor/ProcessTable";
import { SelectionActionBar } from "./ResourceMonitor/SelectionActionBar";
import { Toolbar } from "./ResourceMonitor/Toolbar";
import { processColumns } from "./ResourceMonitor/columns";
import { groupProcessesByApp } from "./ResourceMonitor/groupProcesses";
import type { ProcessRow } from "./ResourceMonitor/types";
import { useSystemSnapshot } from "./ResourceMonitor/useSystemSnapshot";

function ResourceMonitor() {
  const snapshot = useSystemSnapshot();
  const [sorting, setSorting] = useState<SortingState>([
    { id: "cpu_usage_percent", desc: true },
  ]);
  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [expanded, setExpanded] = useState<ExpandedState>({});
  const [selectedPid, setSelectedPid] = useState<number | null>(null);

  const appRows = useMemo(
    () => groupProcessesByApp(snapshot.processes),
    [snapshot.processes],
  );

  const table = useReactTable({
    data: appRows,
    columns: processColumns,
    state: { sorting, columnVisibility, expanded },
    onSortingChange: setSorting,
    onColumnVisibilityChange: setColumnVisibility,
    onExpandedChange: setExpanded,
    getSubRows: (row) => row.subRows,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getExpandedRowModel: getExpandedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    initialState: { pagination: { pageSize: 40 } },
    autoResetPageIndex: false,
    autoResetExpanded: false,
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
      <Toolbar table={table} snapshot={snapshot} />
      <ProcessTable table={table} selectedPid={selectedPid} onSelectRow={handleSelectRow} />
      {selectedProcess && (
        <SelectionActionBar
          process={selectedProcess}
          onViewMore={handleViewMore}
          onTrack={handleTrack}
        />
      )}
      <PaginationBar table={table} totalCount={appRows.length} />
    </div>
  );
}

export default ResourceMonitor;
