import { ChartSpline, SearchX } from "lucide-react";
import { useMemo, useState } from "react";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsContent } from "@/components/ui/tabs";
import { saveCsv } from "@/lib/exportCsv";
import { formatBytes, formatPercent, formatRate } from "./ResourceMonitor/format";
import {
  CPU_COLOR,
  DISK_READ_COLOR,
  DISK_WRITE_COLOR,
  MEMORY_COLOR,
  RANGE_OPTIONS,
  type MetricView,
  type RangeKey,
} from "./Track/constants";
import { cpuColumns, memoryColumns, storageColumns } from "./Track/columns";
import { LegendDots, MetricSection } from "./Track/MetricSection";
import { MetricChart } from "./Track/MetricChart";
import { MetricToolbar } from "./Track/MetricToolbar";
import { formatBytesTick, formatPercentTick, formatRateTick } from "./Track/format";
import { buildStorageSeries, buildTotalSeries, bucketSamplesForTable } from "./Track/series";
import { computeSeriesStats } from "./Track/stats";
import { TrackHeader } from "./Track/TrackHeader";
import { useMetricTable } from "./Track/useMetricTable";
import { useTrackedSession } from "./Track/useTrackedSession";

const PAGE_ROOT =
  "flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm";

const PANEL = "flex min-h-0 flex-1 flex-col";

function Track({ tabId }: { tabId: string }) {
  const trackedId = Number(tabId.replace(/^track-/, ""));
  const [range, setRange] = useState<RangeKey>("10m");
  const [view, setView] = useState<MetricView>("cpu");

  const { tracked, notFound, samples, stopTracking } = useTrackedSession(trackedId, range);

  const rangeLabel = RANGE_OPTIONS[range].label;

  const tableRows = useMemo(() => bucketSamplesForTable(samples), [samples]);
  const cpuSeries = useMemo(() => buildTotalSeries(samples, "cpu_usage_percent"), [samples]);
  const memSeries = useMemo(() => buildTotalSeries(samples, "memory_bytes"), [samples]);
  const storageSeries = useMemo(() => buildStorageSeries(samples), [samples]);

  const cpuStats = useMemo(
    () => computeSeriesStats(cpuSeries.map((point) => point.total)),
    [cpuSeries],
  );
  const memStats = useMemo(
    () => computeSeriesStats(memSeries.map((point) => point.total)),
    [memSeries],
  );
  const storageStats = useMemo(
    () => ({
      read: computeSeriesStats(storageSeries.map((point) => point.read)),
      write: computeSeriesStats(storageSeries.map((point) => point.write)),
      combined: computeSeriesStats(storageSeries.map((point) => point.read + point.write)),
    }),
    [storageSeries],
  );

  const cpuTable = useMetricTable(tableRows, cpuColumns);
  const memoryTable = useMetricTable(tableRows, memoryColumns);
  const storageTable = useMetricTable(tableRows, storageColumns);

  const handleExport = async () => {
    if (!tracked) return;
    const rows = samples.map((s) => [
      new Date(s.timestamp_ms).toISOString(),
      String(s.pid),
      s.name,
      s.cpu_usage_percent.toFixed(1),
      String(s.memory_bytes),
      String(s.disk_read_bytes_per_sec),
      String(s.disk_written_bytes_per_sec),
    ]);
    await saveCsv(
      `${tracked.name}-track-${tracked.id}.csv`,
      ["Timestamp", "PID", "Name", "CPU %", "Memory Bytes", "Disk Read B/s", "Disk Write B/s"],
      rows,
    );
  };

  if (notFound) {
    return (
      <div className={PAGE_ROOT}>
        <Empty className="h-full">
          <EmptyHeader>
            <EmptyMedia variant="icon">
              <SearchX />
            </EmptyMedia>
            <EmptyTitle>Session not found</EmptyTitle>
            <EmptyDescription>
              This tracking session no longer exists. It may have been deleted.
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      </div>
    );
  }

  if (!tracked) {
    return (
      <div className={PAGE_ROOT}>
        <div className="flex shrink-0 items-center gap-3 border-b border-teal px-4 py-3">
          <Skeleton className="h-9 w-9 rounded-lg" />
          <div className="flex flex-col gap-1.5">
            <Skeleton className="h-4 w-40" />
            <Skeleton className="h-3 w-24" />
          </div>
        </div>
        <div className="flex min-h-0 flex-1 flex-col gap-4 p-4">
          <div className="grid shrink-0 gap-3 sm:grid-cols-3">
            <Skeleton className="h-20" />
            <Skeleton className="h-20" />
            <Skeleton className="h-20" />
          </div>
          <Skeleton className="min-h-0 flex-1" />
        </div>
      </div>
    );
  }

  return (
    <div className={PAGE_ROOT}>
      <TrackHeader
        name={tracked.name}
        pids={tracked.pids}
        isActive={tracked.status === "active"}
        onExport={handleExport}
        exportDisabled={samples.length === 0}
        onStop={stopTracking}
      />

      <Tabs
        value={view}
        onValueChange={(value) => setView(value as MetricView)}
        className="min-h-0 flex-1 gap-0 overflow-hidden"
      >
        <MetricToolbar range={range} onRangeChange={setRange} />

        <div className="flex min-h-0 flex-1 flex-col overflow-y-auto p-4">
          {samples.length === 0 ? (
            <Empty className="h-full border border-dashed border-teal/60">
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <ChartSpline />
                </EmptyMedia>
                <EmptyTitle>No samples yet</EmptyTitle>
                <EmptyDescription>
                  Nothing was recorded in the last {rangeLabel}. Try a wider time range.
                </EmptyDescription>
              </EmptyHeader>
            </Empty>
          ) : (
            <>
              <TabsContent value="cpu" className={PANEL}>
                <MetricSection
                  title="CPU usage"
                  stats={[
                    { label: "Current", value: formatPercent(cpuStats.current) },
                    { label: "Average", value: formatPercent(cpuStats.average) },
                    { label: "Peak", value: formatPercent(cpuStats.peak) },
                  ]}
                  table={cpuTable}
                  rowCount={tableRows.length}
                  chart={
                    <MetricChart
                      config={{ total: { label: tracked.name, color: CPU_COLOR } }}
                      data={cpuSeries}
                      series={[{ dataKey: "total", color: CPU_COLOR, label: tracked.name }]}
                      yTickFormatter={formatPercentTick}
                      valueFormatter={formatPercent}
                      yAxisWidth={44}
                    />
                  }
                />
              </TabsContent>

              <TabsContent value="memory" className={PANEL}>
                <MetricSection
                  title="Memory usage"
                  stats={[
                    { label: "Current", value: formatBytes(memStats.current) },
                    { label: "Average", value: formatBytes(memStats.average) },
                    { label: "Peak", value: formatBytes(memStats.peak) },
                  ]}
                  table={memoryTable}
                  rowCount={tableRows.length}
                  chart={
                    <MetricChart
                      config={{ total: { label: tracked.name, color: MEMORY_COLOR } }}
                      data={memSeries}
                      series={[{ dataKey: "total", color: MEMORY_COLOR, label: tracked.name }]}
                      yTickFormatter={formatBytesTick}
                      valueFormatter={formatBytes}
                    />
                  }
                />
              </TabsContent>

              <TabsContent value="storage" className={PANEL}>
                <MetricSection
                  title="Storage operations"
                  stats={[
                    {
                      label: "Read",
                      value: formatRate(storageStats.read.current),
                      accent: DISK_READ_COLOR,
                    },
                    {
                      label: "Write",
                      value: formatRate(storageStats.write.current),
                      accent: DISK_WRITE_COLOR,
                    },
                    { label: "Peak I/O", value: formatRate(storageStats.combined.peak) },
                  ]}
                  legend={
                    <LegendDots
                      items={[
                        { label: "Read", color: DISK_READ_COLOR },
                        { label: "Write", color: DISK_WRITE_COLOR },
                      ]}
                    />
                  }
                  table={storageTable}
                  rowCount={tableRows.length}
                  chart={
                    <MetricChart
                      config={{
                        read: { label: "Read", color: DISK_READ_COLOR },
                        write: { label: "Write", color: DISK_WRITE_COLOR },
                      }}
                      data={storageSeries}
                      series={[
                        { dataKey: "read", color: DISK_READ_COLOR, label: "Read" },
                        { dataKey: "write", color: DISK_WRITE_COLOR, label: "Write" },
                      ]}
                      yTickFormatter={formatRateTick}
                      valueFormatter={formatRate}
                    />
                  }
                />
              </TabsContent>
            </>
          )}
        </div>
      </Tabs>
    </div>
  );
}

export default Track;
