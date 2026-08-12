import type { Table } from "@tanstack/react-table";
import type { ReactNode } from "react";
import {
  Card,
  CardAction,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { ProcessSamplePoint } from "../ResourceMonitor/types";
import { DataTable } from "./DataTable";
import { PaginationBar } from "./PaginationBar";
import { StatCards, type StatSpec } from "./StatCards";

/** Everything shown for one metric view: the summary figures, the chart, and
 * the sample history beneath it. Switching CPU/Memory/Storage swaps this whole
 * block.
 *
 * Both cards run `gap-0 py-0` and pad their own sections instead: `Card`'s
 * default `gap-(--card-spacing)` would otherwise open a second 16px gap under
 * the header rule, on top of the padding the header already carries, and the
 * table's column headers would float away from the panel edge.
 *
 * The table card takes the leftover height and scrolls internally rather than
 * running off the bottom of the page, which is what keeps its sticky header and
 * its pagination on screen no matter how many rows there are. */
export function MetricSection({
  title,
  stats,
  legend,
  chart,
  table,
  rowCount,
}: {
  title: string;
  stats: StatSpec[];
  /** Extra content in the chart card's header, e.g. storage's Read/Write key. */
  legend?: ReactNode;
  chart: ReactNode;
  table: Table<ProcessSamplePoint>;
  rowCount: number;
}) {
  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4">
      <StatCards stats={stats} />

      <Card className="shrink-0 gap-0 rounded-lg py-0 ring-teal/50">
        <CardHeader className="border-b pt-4">
          <CardTitle className="text-sm">{title}</CardTitle>
          {legend && <CardAction>{legend}</CardAction>}
        </CardHeader>
        <CardContent className="p-4">{chart}</CardContent>
      </Card>

      <Card className="min-h-96 flex-1 gap-0 rounded-lg py-0 ring-teal/50">
        <CardHeader className="shrink-0 border-b pt-4">
          <CardTitle className="text-sm">Process data</CardTitle>
        </CardHeader>
        <CardContent className="min-h-0 flex-1 p-0">
          <DataTable table={table} />
        </CardContent>
        <CardFooter className="shrink-0 px-4 py-2.5">
          <PaginationBar table={table} totalCount={rowCount} />
        </CardFooter>
      </Card>
      <div className="border-4 border-transparent"/>
    </div>
  );
}

/** Small color key for a chart card header, e.g. "● Read  ● Write". */
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
