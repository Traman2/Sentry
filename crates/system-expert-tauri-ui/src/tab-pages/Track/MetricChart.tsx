import { useId } from "react";
import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from "recharts";
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from "@/components/ui/chart";
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
  // Gradient ids have to be unique per mounted chart, or two charts on screen
  // would share (and fight over) the same <defs> entry.
  const gradientId = useId().replace(/:/g, "");
  const labelByKey = Object.fromEntries(series.map((s) => [s.dataKey, s.label]));

  return (
    <ChartContainer config={config} className="aspect-auto h-64 w-full">
      <AreaChart data={data} margin={{ top: 8, right: 8, bottom: 0, left: 0 }}>
        <defs>
          {series.map((s) => (
            <linearGradient
              key={s.dataKey}
              id={`${gradientId}-${s.dataKey}`}
              x1="0"
              y1="0"
              x2="0"
              y2="1"
            >
              <stop offset="5%" stopColor={s.color} stopOpacity={0.3} />
              <stop offset="95%" stopColor={s.color} stopOpacity={0.02} />
            </linearGradient>
          ))}
        </defs>
        <CartesianGrid vertical={false} strokeDasharray="3 3" />
        <XAxis
          dataKey="timestamp_ms"
          type="number"
          scale="time"
          domain={["dataMin", "dataMax"]}
          tickFormatter={formatTick}
          tickLine={false}
          axisLine={false}
          tickMargin={8}
          minTickGap={28}
          tick={{ fontSize: 11 }}
        />
        <YAxis
          domain={[0, "auto"]}
          tickFormatter={yTickFormatter}
          tickLine={false}
          axisLine={false}
          width={yAxisWidth}
          tickMargin={4}
          tick={{ fontSize: 11 }}
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
            fill={`url(#${gradientId}-${s.dataKey})`}
            strokeWidth={2}
            activeDot={{ r: 3, strokeWidth: 0 }}
            connectNulls
            isAnimationActive={false}
          />
        ))}
      </AreaChart>
    </ChartContainer>
  );
}
