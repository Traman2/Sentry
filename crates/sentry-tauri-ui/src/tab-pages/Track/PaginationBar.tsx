import type { Table } from "@tanstack/react-table";
import { ChevronLeft, ChevronRight } from "lucide-react";
import type { ProcessSamplePoint } from "../ResourceMonitor/types";

/** Same chrome as `ResourceMonitor/PaginationBar.tsx` — the 5-second-bucketed
 * history table can have thousands of rows at the 24hr range, so it needs
 * paging just as much as the process table does. */
export function PaginationBar({
  table,
  totalCount,
}: {
  table: Table<ProcessSamplePoint>;
  totalCount: number;
}) {
  return (
    <div className="flex items-center justify-between px-1 py-2 text-[11px] text-muted-foreground">
      <span>
        Page {table.getState().pagination.pageIndex + 1} of{" "}
        {Math.max(table.getPageCount(), 1)} · {totalCount} rows
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
  );
}
