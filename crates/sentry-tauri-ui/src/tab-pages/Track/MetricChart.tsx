import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from "recharts";
import { ChartContainer, ChartTooltip, ChartTooltipContent, type ChartConfig } from "@/components/ui/chart";
import { formatTick } from "./format";

export interface ChartSeriesSpec {
  dataKey: string;
  color: string;
  /** Tooltip row label for this series, e.g. the process name for a single
   * combined line, or "Read"/"Write" for the two-line storage chart. */
  label: string;
}

/** One area chart, generalized over the three metric views: a single "total"
 * line (CPU, Memory) or multiple named lines (Storage's Read/Write). */
export function MetricChart({
  config,
  data,
  series,
  yTickFormatter,
  valueFormatter,
  yAxisWidth = 64,
}: {
  config: ChartConfig;
  data: Record<string, number>[];
  series: ChartSeriesSpec[];
  yTickFormatter: (value: number) => string;
  valueFormatter: (value: number) => string;
  yAxisWidth?: number;
}) {
  const labelByKey = Object.fromEntries(series.map((s) => [s.dataKey, s.label]));

  return (
    <ChartContainer config={config} className="aspect-auto h-64 w-full">
      <AreaChart data={data}>
        <CartesianGrid vertical={false} />
        <XAxis
          dataKey="timestamp_ms"
          type="number"
          scale="time"
          domain={["dataMin", "dataMax"]}
          tickFormatter={formatTick}
          tickLine={false}
          axisLine={false}
        />
        <YAxis
          domain={[0, "auto"]}
          tickFormatter={yTickFormatter}
          tickLine={false}
          axisLine={false}
          width={yAxisWidth}
        />
        <ChartTooltip
          content={
            <ChartTooltipContent
              labelFormatter={(_, payload) => formatTick(payload?.[0]?.payload?.timestamp_ms ?? 0)}
              formatter={(value, name) => (
                <div className="flex w-full items-center justify-between gap-3">
                  <span className="text-muted-foreground">
                    {labelByKey[String(name)] ?? String(name)}
                  </span>
                  <span className="font-mono font-medium text-navy tabular-nums">
                    {valueFormatter(Number(value))}
                  </span>
                </div>
              )}
            />
          }
        />
        {series.map((s) => (
          <Area
            key={s.dataKey}
            dataKey={s.dataKey}
            type="monotone"
            stroke={s.color}
            fill={s.color}
            fillOpacity={series.length > 1 ? 0.1 : 0.15}
            strokeWidth={1.5}
            connectNulls
            isAnimationActive={false}
          />
        ))}
      </AreaChart>
    </ChartContainer>
  );
}
