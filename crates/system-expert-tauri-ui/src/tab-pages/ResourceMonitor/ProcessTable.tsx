import { type Table as TanstackTable, flexRender } from "@tanstack/react-table";
import { SortIcon } from "@/components/SortIcon";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { cn } from "@/lib/utils";
import type { AppRow } from "./types";

/** The process list, filling the page between the toolbar and the pagination
 * bar. The container is the scroll region and the header cells are sticky
 * against it, so column labels survive scrolling a long list. That needs
 * `border-separate`: under the default collapsed model the browser hands
 * borders to the table itself and a sticky header loses its underline. */
export function ProcessTable({
  table,
  selectedPid,
  onSelectRow,
}: {
  table: TanstackTable<AppRow>;
  selectedPid: number | null;
  onSelectRow: (pid: number) => void;
}) {
  return (
    <Table
      containerClassName="min-h-0 flex-1"
      className="border-separate border-spacing-0"
      style={{ tableLayout: "fixed" }}
    >
      <colgroup>
        {table.getFlatHeaders().map((header) => (
          <col key={header.id} style={{ width: header.getSize() }} />
        ))}
      </colgroup>
      <TableHeader className="[&_tr]:border-b-0">
        {table.getHeaderGroups().map((headerGroup) => (
          <TableRow key={headerGroup.id} className="hover:bg-transparent">
            {headerGroup.headers.map((header) => (
              <TableHead
                key={header.id}
                className="sticky top-0 z-10 h-9 border-b border-teal bg-muted px-4 text-xs font-medium text-navy/60"
              >
                {header.isPlaceholder ? null : header.column.getCanSort() ? (
                  <button
                    type="button"
                    onClick={header.column.getToggleSortingHandler()}
                    className="flex cursor-pointer items-center gap-1.5 hover:text-navy"
                  >
                    {flexRender(header.column.columnDef.header, header.getContext())}
                    <SortIcon direction={header.column.getIsSorted()} />
                  </button>
                ) : (
                  flexRender(header.column.columnDef.header, header.getContext())
                )}
              </TableHead>
            ))}
          </TableRow>
        ))}
      </TableHeader>
      <TableBody>
        {table.getRowModel().rows.map((row) => (
          <TableRow
            key={row.id}
            onClick={() => onSelectRow(row.original.pid)}
            className={cn(
              "cursor-pointer border-0 hover:bg-teal/10",
              selectedPid === row.original.pid && "bg-teal/20 hover:bg-teal/20",
            )}
          >
            {row.getVisibleCells().map((cell) => (
              <TableCell
                key={cell.id}
                className="overflow-hidden border-b border-teal/25 px-4 py-2.5"
              >
                {flexRender(cell.column.columnDef.cell, cell.getContext())}
              </TableCell>
            ))}
          </TableRow>
        ))}
      </TableBody>
    </Table>
  );
}
