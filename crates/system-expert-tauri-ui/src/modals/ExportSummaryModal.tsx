import { useMemo } from "react";
import { DialogClose, DialogTitle } from "@/components/ui/dialog";
import { saveCsv } from "@/lib/exportCsv";
import { groupProcessesByApp } from "@/tab-pages/ResourceMonitor/groupProcesses";
import type { SystemSnapshot } from "@/tab-pages/ResourceMonitor/types";

function StatRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="flex items-center justify-between text-sm">
      <span className="text-navy/60">{label}</span>
      <span className="font-medium text-navy">{value}</span>
    </div>
  );
}

function ExportSummaryModal({ snapshot }: { snapshot: SystemSnapshot }) {
  const { grouped, totalApps, totalProcesses, topConsumer } = useMemo(() => {
    const grouped = groupProcessesByApp(snapshot.processes);
    const sorted = grouped.slice().sort((a, b) => b.cpu_usage_percent - a.cpu_usage_percent);
    return {
      grouped,
      totalApps: grouped.length,
      totalProcesses: snapshot.processes.length,
      topConsumer: sorted[0] ?? null,
    };
  }, [snapshot]);

  const handleExport = async () => {
    const timestamp = new Date(snapshot.timestamp_ms).toISOString().replace(/[:.]/g, "-");
    await saveCsv(
      `resource-snapshot-${timestamp}.csv`,
      ["Name", "PID Count", "CPU %", "Memory", "Disk Read/s", "Disk Write/s"],
      grouped.map((app) => [
        app.name,
        String(app.pid_count),
        app.cpu_usage_display,
        app.memory_display,
        app.disk_read_display,
        app.disk_write_display,
      ]),
    );
  };

  return (
    <div className="w-96 rounded-lg border border-teal bg-canvas p-5 shadow-lg">
      <DialogTitle>Export Summary</DialogTitle>
      <p className="mt-1 text-xs text-muted-foreground">
        Snapshot taken {new Date(snapshot.timestamp_ms).toLocaleTimeString()}
      </p>

      <div className="mt-4 flex flex-col gap-2 rounded-md border border-teal/50 bg-teal/5 p-3">
        <StatRow label="Total apps" value={String(totalApps)} />
        <StatRow label="Total processes" value={String(totalProcesses)} />
        <StatRow
          label="Top consumer"
          value={topConsumer ? `${topConsumer.name} (${topConsumer.cpu_usage_display})` : "—"}
        />
        {topConsumer && (
          <StatRow label="Its memory usage" value={topConsumer.memory_display} />
        )}
      </div>

      <div className="mt-5 flex items-center justify-end gap-2">
        <DialogClose className="cursor-pointer rounded-md px-3 py-1.5 text-sm text-navy/70 hover:bg-teal/25">
          Close
        </DialogClose>
        <button
          onClick={handleExport}
          className="cursor-pointer rounded-md bg-navy px-3 py-1.5 text-sm font-medium text-canvas hover:bg-navy/80"
        >
          Export CSV
        </button>
      </div>
    </div>
  );
}

export default ExportSummaryModal;