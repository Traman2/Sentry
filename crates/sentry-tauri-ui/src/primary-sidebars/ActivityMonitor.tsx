import { Activity, ArrowDown, ArrowUp, Cpu, Flame, MemoryStick } from "lucide-react";
import { useMemo } from "react";
import type { ReactNode } from "react";
import { PanelSection } from "@/components/PanelList";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { healthColor } from "@/lib/health";
import { useTabStore, type Tab } from "@/store/tabs";
import { formatBytes, formatRate } from "@/tab-pages/ResourceMonitor/format";
import { groupProcessesByApp } from "@/tab-pages/ResourceMonitor/groupProcesses";
import { useSystemSnapshot } from "@/tab-pages/ResourceMonitor/useSystemSnapshot";

const RESOURCE_MONITOR_TAB: Tab = {
  id: "resource-monitor",
  type: "resource-monitor",
  title: "Resource Monitor",
};

function StatCard({
  icon,
  label,
  children,
}: {
  icon: ReactNode;
  label: string;
  children: ReactNode;
}) {
  return (
    <Card size="sm" className="rounded-lg ring-teal/50 transition-shadow hover:shadow-sm">
      <CardHeader>
        <CardTitle className="flex items-center gap-1.5 text-[10px] font-semibold tracking-wide text-navy/60 uppercase">
          {icon}
          {label}
        </CardTitle>
      </CardHeader>
      <CardContent>{children}</CardContent>
    </Card>
  );
}

function UsageBar({ percent }: { percent: number }) {
  const clamped = Math.min(Math.max(percent, 0), 100);
  return (
    <div className="h-1.5 w-full overflow-hidden rounded-full bg-navy/10">
      <div
        className="h-full rounded-full transition-[width] duration-500"
        style={{ width: `${clamped}%`, backgroundColor: healthColor(percent) }}
      />
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
    <div className="flex flex-col gap-3">
      <Button size="sm" className="w-full" onClick={() => openTab(RESOURCE_MONITOR_TAB)}>
        <Activity />
        Open Activity Monitor
      </Button>

      <PanelSection label="Live overview" bodyClassName="gap-1.5">
        <StatCard icon={<Cpu className="h-3 w-3" />} label="CPU">
          <div className="flex items-baseline justify-between">
            <span className="font-heading text-lg font-semibold text-navy tabular-nums">
              {cpuPercent.toFixed(1)}%
            </span>
          </div>
          <div className="mt-1.5">
            <UsageBar percent={cpuPercent} />
          </div>
        </StatCard>

        <StatCard icon={<MemoryStick className="h-3 w-3" />} label="Memory">
          <div className="flex items-baseline justify-between">
            <span className="font-heading text-lg font-semibold text-navy tabular-nums">
              {memoryPercent.toFixed(1)}%
            </span>
            <span className="text-[11px] text-muted-foreground tabular-nums">
              {formatBytes(memoryBytes)}
            </span>
          </div>
          <div className="mt-1.5">
            <UsageBar percent={memoryPercent} />
          </div>
        </StatCard>

        <StatCard icon={<Activity className="h-3 w-3" />} label="Network">
          <div className="flex flex-col gap-1.5">
            <div className="flex items-center justify-between">
              <span className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
                <ArrowDown className="h-3 w-3 text-teal" />
                Down
              </span>
              <span className="text-xs font-medium text-navy tabular-nums">
                {formatRate(networkDown)}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
                <ArrowUp className="h-3 w-3 text-teal" />
                Up
              </span>
              <span className="text-xs font-medium text-navy tabular-nums">
                {formatRate(networkUp)}
              </span>
            </div>
          </div>
        </StatCard>

        {topConsumer && (
          <StatCard icon={<Flame className="h-3 w-3" />} label="Top consumer">
            <div className="truncate text-xs font-medium text-navy">{topConsumer.name}</div>
            <div className="mt-1 flex items-center justify-between text-[11px] text-muted-foreground">
              <span className="tabular-nums">{topConsumer.cpu_usage_display} CPU</span>
              <span className="tabular-nums">{topConsumer.memory_display}</span>
            </div>
            <div className="mt-1.5">
              <UsageBar percent={topConsumer.cpu_usage_percent} />
            </div>
          </StatCard>
        )}
      </PanelSection>
    </div>
  );
}

export default ActivityMonitor;
