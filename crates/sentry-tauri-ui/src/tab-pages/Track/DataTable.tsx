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
import type { ProcessSamplePoint } from "../ResourceMonitor/types";

/** The bucketed sample history under each chart, scoped to the fixed set of
 * pids being tracked (one row per pid — a tracked "app" is never re-grouped,
 * so there's no expand/collapse here).
 *
 * The container is the scroll region and the header cells are sticky against
 * it, so column labels stay readable all the way down. That needs
 * `border-separate`: under the default collapsed model the browser hands
 * borders to the table itself and a sticky header loses its underline. */
export function DataTable({ table }: { table: TanstackTable<ProcessSamplePoint> }) {
  const rows = table.getRowModel().rows;

  return (
    <Table
      containerClassName="h-full"
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
      <TableBody className="[&_tr:last-child>td]:border-b-0">
        {rows.map((row) => (
          <TableRow key={row.id} className="border-0 hover:bg-teal/10">
            {row.getVisibleCells().map((cell) => (
              <TableCell
                key={cell.id}
                className="overflow-hidden border-b border-teal/25 px-4 py-3"
              >
                {flexRender(cell.column.columnDef.cell, cell.getContext())}
              </TableCell>
            ))}
          </TableRow>
        ))}
        {rows.length === 0 && (
          <TableRow className="border-0 hover:bg-transparent">
            <TableCell
              colSpan={table.getFlatHeaders().length}
              className="h-24 text-center text-xs text-muted-foreground"
            >
              No data yet.
            </TableCell>
          </TableRow>
        )}
      </TableBody>
    </Table>
  );
}
