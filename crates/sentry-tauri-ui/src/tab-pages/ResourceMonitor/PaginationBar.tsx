import type { Table } from "@tanstack/react-table";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import type { AppRow } from "./types";

/** Pinned below the process list — outside the table's scroll region, so it
 * stays put however far down the list you are. */
export function PaginationBar({
  table,
  totalCount,
}: {
  table: Table<AppRow>;
  totalCount: number;
}) {
  const { pageIndex } = table.getState().pagination;

  return (
    <div className="flex shrink-0 items-center justify-between gap-2 border-t border-teal px-4 py-2">
      <span className="text-[11px] text-muted-foreground tabular-nums">
        Page {pageIndex + 1} of {Math.max(table.getPageCount(), 1)} · {totalCount} apps
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
