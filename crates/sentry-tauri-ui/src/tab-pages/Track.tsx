import { useMemo, useState } from "react";
import { saveCsv } from "@/lib/exportCsv";
import { formatBytes, formatPercent, formatRate } from "./ResourceMonitor/format";
import {
  CPU_COLOR,
  DISK_READ_COLOR,
  DISK_WRITE_COLOR,
  MEMORY_COLOR,
  type MetricView,
  type RangeKey,
} from "./Track/constants";
import { cpuColumns, memoryColumns, storageColumns } from "./Track/columns";
import { EndedBanner } from "./Track/EndedBanner";
import { LegendDots, MetricSection } from "./Track/MetricSection";
import { MetricChart } from "./Track/MetricChart";
import { formatBytesTick, formatPercentTick, formatRateTick } from "./Track/format";
import { buildStorageSeries, buildTotalSeries, bucketSamplesForTable } from "./Track/series";
import { TrackHeader } from "./Track/TrackHeader";
import { useMetricTable } from "./Track/useMetricTable";
import { useTrackedSession } from "./Track/useTrackedSession";

function Track({ tabId }: { tabId: string }) {
  const trackedId = Number(tabId.replace(/^track-/, ""));
  const [range, setRange] = useState<RangeKey>("10m");
  const [view, setView] = useState<MetricView>("cpu");

  const { tracked, notFound, samples, endReason, stopTracking } = useTrackedSession(
    trackedId,
    range,
  );

  const tableRows = useMemo(() => bucketSamplesForTable(samples), [samples]);
  const cpuSeries = useMemo(() => buildTotalSeries(samples, "cpu_usage_percent"), [samples]);
  const memSeries = useMemo(() => buildTotalSeries(samples, "memory_bytes"), [samples]);
  const storageSeries = useMemo(() => buildStorageSeries(samples), [samples]);

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
      <div className="flex h-full w-full items-center justify-center rounded-lg border border-teal bg-canvas p-6 text-sm text-muted-foreground shadow-sm">
        This tracking session no longer exists. It may have been deleted.
      </div>
    );
  }

  if (!tracked) {
    return (
      <div className="flex h-full w-full items-center justify-center rounded-lg border border-teal bg-canvas p-6 text-sm text-muted-foreground shadow-sm">
        Loading…
      </div>
    );
  }

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm">
      <TrackHeader
        name={tracked.name}
        pids={tracked.pids}
        view={view}
        onViewChange={setView}
        range={range}
        onRangeChange={setRange}
        onExport={handleExport}
        exportDisabled={samples.length === 0}
        showStop={tracked.status === "active"}
        onStop={stopTracking}
      />

      {tracked.status === "ended" && <EndedBanner reason={endReason} />}

      <div className="flex-1 overflow-y-auto p-4">
        <div className="flex flex-col gap-4">
          {view === "cpu" && (
            <MetricSection
              title="CPU usage"
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
          )}

          {view === "memory" && (
            <MetricSection
              title="Memory usage"
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
          )}

          {view === "storage" && (
            <MetricSection
              title="Storage operations"
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
          )}

          {samples.length === 0 && (
            <p className="text-center text-sm text-muted-foreground">
              No samples in this window yet.
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

export default Track;
