import { type Table, flexRender } from "@tanstack/react-table";
import { ArrowUpDown } from "lucide-react";
import type { ProcessSamplePoint } from "../ResourceMonitor/types";

/** Same table chrome as `ResourceMonitor/ProcessTable.tsx`, scoped to the fixed
 * set of pids being tracked (one row per pid — a tracked "app" is never
 * re-grouped, so there's no expand/collapse here). */
export function DataTable({ table }: { table: Table<ProcessSamplePoint> }) {
  return (
    <div className="overflow-x-auto rounded-lg border border-teal">
      <table
        className="w-full text-left"
        style={{ tableLayout: "fixed", borderCollapse: "collapse" }}
      >
        <colgroup>
          {table.getFlatHeaders().map((header) => (
            <col key={header.id} style={{ width: header.getSize() }} />
          ))}
        </colgroup>
        <thead className="bg-teal/10">
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
                      {header.column.getCanSort() && <ArrowUpDown className="h-3 w-3" />}
                    </button>
                  )}
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map((row) => (
            <tr key={row.id} className="border-b border-teal/50 last:border-b-0 hover:bg-teal/10">
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
          {table.getRowModel().rows.length === 0 && (
            <tr>
              <td
                colSpan={table.getFlatHeaders().length}
                className="px-4 py-3 text-center text-xs text-muted-foreground"
              >
                No data yet.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}
