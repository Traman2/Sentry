import type { Table } from "@tanstack/react-table";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import type { ProcessSamplePoint } from "../ResourceMonitor/types";

/** Paging for the sample history table — at the 24hr range the 5-second-bucketed
 * rows run into the thousands, so this needs paging just as much as the process
 * table does. Rendered inside the table card's footer. */
export function PaginationBar({
  table,
  totalCount,
}: {
  table: Table<ProcessSamplePoint>;
  totalCount: number;
}) {
  const { pageIndex } = table.getState().pagination;

  return (
    <div className="flex w-full items-center justify-between gap-2">
      <span className="text-[11px] text-muted-foreground tabular-nums">
        Page {pageIndex + 1} of {Math.max(table.getPageCount(), 1)} · {totalCount} rows
      </span>
      <ButtonGroup>
        <Button
          variant="outline"
          size="icon-sm"
          aria-label="Previous page"
          onClick={() => table.previousPage()}
          disabled={!table.getCanPreviousPage()}
        >
          <ChevronLeft />
        </Button>
        <Button
          variant="outline"
          size="icon-sm"
          aria-label="Next page"
          onClick={() => table.nextPage()}
          disabled={!table.getCanNextPage()}
        >
          <ChevronRight />
        </Button>
      </ButtonGroup>
    </div>
  );
}
