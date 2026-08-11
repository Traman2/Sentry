import type { Table } from "@tanstack/react-table";
import type { ReactNode } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import type { ProcessSamplePoint } from "../ResourceMonitor/types";
import { DataTable } from "./DataTable";
import { PaginationBar } from "./PaginationBar";

/** A chart card plus its underlying "Process data" table — one per metric
 * view, so switching CPU/Memory/Storage swaps this whole block. */
export function MetricSection({
  title,
  legend,
  chart,
  table,
  rowCount,
}: {
  title: string;
  /** Extra content in the card header, e.g. storage's Read/Write color key. */
  legend?: ReactNode;
  chart: ReactNode;
  table: Table<ProcessSamplePoint>;
  rowCount: number;
}) {
  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle className={legend ? "flex items-center justify-between" : undefined}>
            {title}
            {legend}
          </CardTitle>
        </CardHeader>
        <CardContent>{chart}</CardContent>
      </Card>

      <div>
        <h3 className="mb-1.5 text-[10px] font-semibold uppercase tracking-wide text-navy/60">
          Process data
        </h3>
        <DataTable table={table} />
        <PaginationBar table={table} totalCount={rowCount} />
      </div>
    </>
  );
}

/** Small color-key legend for a card header, e.g. "● Read  ● Write". */
export function LegendDots({ items }: { items: { label: string; color: string }[] }) {
  return (
    <span className="flex items-center gap-3 text-xs font-normal text-muted-foreground">
      {items.map((item) => (
        <span key={item.label} className="flex items-center gap-1.5">
          <span className="h-2 w-2 rounded-full" style={{ backgroundColor: item.color }} />
          {item.label}
        </span>
      ))}
    </span>
  );
}
