import { ArrowDown, ArrowUp } from "lucide-react";
import { useMemo } from "react";
import type { ReactNode } from "react";
import { Button } from "@/components/ui/button";
import Gauge from "@/components/Gauge";
import { useTabStore, type Tab } from "@/store/tabs";
import { formatBytes, formatRate } from "@/tab-pages/ResourceMonitor/format";
import { groupProcessesByApp } from "@/tab-pages/ResourceMonitor/groupProcesses";
import { useSystemSnapshot } from "@/tab-pages/ResourceMonitor/useSystemSnapshot";

const RESOURCE_MONITOR_TAB: Tab = {
  id: "resource-monitor",
  type: "resource-monitor",
  title: "Resource Monitor",
};

function Card({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="rounded-lg border border-teal/50 p-3">
      <div className="text-[10px] font-semibold uppercase tracking-wide text-navy/60">
        {label}
      </div>
      <div className="mt-2">{children}</div>
    </div>
  );
}

function ActivityMonitor() {
  const openTab = useTabStore((state) => state.openTab);
  const snapshot = useSystemSnapshot();

  const { cpuPercent, memoryPercent, memoryBytes, networkDown, networkUp, topConsumer } =
    useMemo(() => {
      const cpuPercent = snapshot.processes.reduce((sum, p) => sum + p.cpu_usage_percent, 0);
      const memoryPercent = snapshot.processes.reduce((sum, p) => sum + p.memory_percent, 0);
      const memoryBytes = snapshot.processes.reduce((sum, p) => sum + p.memory_bytes, 0);
      const networkDown = snapshot.networks.reduce(
        (sum, n) => sum + n.received_bytes_per_sec,
        0,
      );
      const networkUp = snapshot.networks.reduce(
        (sum, n) => sum + n.transmitted_bytes_per_sec,
        0,
      );
      const topConsumer = groupProcessesByApp(snapshot.processes)
        .slice()
        .sort((a, b) => b.cpu_usage_percent - a.cpu_usage_percent)[0];

      return { cpuPercent, memoryPercent, memoryBytes, networkDown, networkUp, topConsumer };
    }, [snapshot]);

  return (
    <div className="flex flex-col gap-1">
      <p className="text-xs text-muted-foreground">View active apps and processes</p>
      <Button className="rounded-lg" onClick={() => openTab(RESOURCE_MONITOR_TAB)}>
        Open Activity Monitor
      </Button>

      <p className="text-xs text-muted-foreground mt-2">More Details</p>
      <Card label="System load">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Gauge percent={cpuPercent} size={14} strokeWidth={2.5} />
            <span className="text-xs text-navy">{cpuPercent.toFixed(1)}% CPU</span>
          </div>
        </div>
        <div className="mt-2 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Gauge percent={memoryPercent} size={14} strokeWidth={2.5} />
            <span className="text-xs text-navy">{memoryPercent.toFixed(1)}% Memory</span>
          </div>
          <span className="text-[11px] text-muted-foreground">
            {formatBytes(memoryBytes)}
          </span>
        </div>
      </Card>

      <Card label="Network">
        <div className="flex flex-col gap-1 text-[11px] text-navy">
          <span className="flex items-center gap-1">
            <ArrowDown className="h-3 w-3 text-navy/40" />
            {formatRate(networkDown)}
          </span>
          <span className="flex items-center gap-1">
            <ArrowUp className="h-3 w-3 text-navy/40" />
            {formatRate(networkUp)}
          </span>
        </div>
      </Card>

      {topConsumer && (
        <Card label="Top consumer">
          <div className="truncate text-xs font-medium text-navy">{topConsumer.name}</div>
          <div className="mt-0.5 flex items-center gap-1.5">
            <Gauge percent={topConsumer.cpu_usage_percent} />
            <span className="text-[11px] text-muted-foreground">
              {topConsumer.cpu_usage_display} CPU · {topConsumer.memory_display}
            </span>
          </div>
        </Card>
      )}
    </div>
  );
}

export default ActivityMonitor;
