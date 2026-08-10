import { type Table, flexRender } from "@tanstack/react-table";
import { ArrowUpDown } from "lucide-react";
import { cn } from "@/lib/utils";
import type { AppRow } from "./types";

export function ProcessTable({
  table,
  selectedPid,
  onSelectRow,
}: {
  table: Table<AppRow>;
  selectedPid: number | null;
  onSelectRow: (pid: number) => void;
}) {
  return (
    <div className="flex-1 overflow-y-auto overflow-x-auto">
      <table
        className="w-full text-left"
        style={{ tableLayout: "fixed", borderCollapse: "collapse" }}
      >
        <colgroup>
          {table.getFlatHeaders().map((header) => (
            <col key={header.id} style={{ width: header.getSize() }} />
          ))}
        </colgroup>
        <thead className="sticky top-0 bg-canvas">
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
            <tr
              key={row.id}
              onClick={() => onSelectRow(row.original.pid)}
              className={cn(
                "cursor-pointer border-b border-teal/50 hover:bg-teal/10",
                selectedPid === row.original.pid && "bg-teal/20 hover:bg-teal/20",
              )}
            >
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
        </tbody>
      </table>
    </div>
  );
}