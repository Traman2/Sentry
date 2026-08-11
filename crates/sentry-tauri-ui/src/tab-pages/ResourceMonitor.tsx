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
import ViewMoreModal from "../modals/ViewMoreModal";
import { useChatStore } from "../store/chat";
import { useModalStore } from "../store/modal";
import { useTabStore } from "../store/tabs";
import { PaginationBar } from "./ResourceMonitor/PaginationBar";
import { ProcessTable } from "./ResourceMonitor/ProcessTable";
import { SelectionActionBar } from "./ResourceMonitor/SelectionActionBar";
import { Toolbar } from "./ResourceMonitor/Toolbar";
import { processColumns } from "./ResourceMonitor/columns";
import { findAppRowByPid, groupProcessesByApp } from "./ResourceMonitor/groupProcesses";
import type { AppRow } from "./ResourceMonitor/types";
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

  const openModal = useModalStore((state) => state.openModal);
  const createChatSpaceWithMessage = useChatStore((state) => state.createChatSpaceWithMessage);
  const openTab = useTabStore((state) => state.openTab);

  const selectedProcess = useMemo(
    () => (selectedPid == null ? null : findAppRowByPid(appRows, selectedPid)),
    [appRows, selectedPid],
  );

  const handleSelectRow = (pid: number) => {
    setSelectedPid((current) => (current === pid ? null : pid));
  };

  const handleViewMore = (row: AppRow) => {
    openModal(<ViewMoreModal row={row} />);
  };

  const handleTrack = (row: AppRow) => {
    console.log("Track", row);
  };

  const handleAskAi = async (row: AppRow) => {
    const detail = await createChatSpaceWithMessage(`Please tell me more about ${row.name}`);
    openTab({ id: String(detail.id), type: "chat-space", title: detail.title });
  };

  const handleKill = (row: AppRow) => {
    console.log("Kill", row);
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
          onAskAi={handleAskAi}
          onKill={handleKill}
        />
      )}
      <PaginationBar table={table} totalCount={appRows.length} />
    </div>
  );
}

export default ResourceMonitor;
