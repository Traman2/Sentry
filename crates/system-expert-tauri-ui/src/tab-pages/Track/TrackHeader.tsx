import { Download, Radar, Square } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

/** The Track page's identity bar: which process/app this session follows and
 * whether it's still live, with the session-level actions (export, stop) on
 * the right. Metric and range controls live in `MetricToolbar` below it. */
export function TrackHeader({
  name,
  pids,
  isActive,
  onExport,
  exportDisabled,
  onStop,
}: {
  name: string;
  pids: number[];
  isActive: boolean;
  onExport: () => void;
  exportDisabled: boolean;
  onStop: () => void;
}) {
  return (
    <div className="flex shrink-0 flex-wrap items-center justify-between gap-3 border-b border-teal px-4 py-3">
      <div className="flex min-w-0 items-center gap-3">
        <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg bg-teal/40 font-heading text-sm font-semibold text-navy">
          {name.slice(0, 1).toUpperCase()}
        </div>
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <h2 className="truncate font-heading text-sm font-semibold text-navy">{name}</h2>
            {isActive ? (
              <Badge variant="secondary">
                <Radar className="animate-pulse" />
                Live
              </Badge>
            ) : (
              <Badge variant="outline">Ended</Badge>
            )}
          </div>
          <p className="truncate text-[11px] text-muted-foreground">
            {pids.length > 1 ? `${pids.length} processes` : `PID ${pids[0]}`}
          </p>
        </div>
      </div>

      <div className="flex items-center gap-2">
        <Button variant="outline" size="sm" onClick={onExport} disabled={exportDisabled}>
          <Download />
          Export CSV
        </Button>
        {isActive && (
          <Button variant="destructive" size="sm" onClick={onStop}>
            <Square />
            Stop tracking
          </Button>
        )}
      </div>
    </div>
  );
}
