import {
  type ColumnDef,
  type SortingState,
  getCoreRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import { useState } from "react";
import type { ProcessSamplePoint } from "../ResourceMonitor/types";

/** One tanstack table instance with its own sorting state, over a fixed
 * column shape. Call once per metric view (cpu/memory/storage) rather than
 * reusing a single table instance across different column shapes — tanstack
 * keeps internal state keyed to a table instance, and swapping `columns` on
 * one shared instance between structurally different sets is what used to
 * crash the app when switching views. */
export function useMetricTable(data: ProcessSamplePoint[], columns: ColumnDef<ProcessSamplePoint>[]) {
  const [sorting, setSorting] = useState<SortingState>([{ id: "timestamp_ms", desc: true }]);
  return useReactTable({
    data,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    initialState: { pagination: { pageSize: 25 } },
  });
}
