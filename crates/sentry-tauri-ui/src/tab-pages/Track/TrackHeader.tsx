import { Cpu, Download, HardDrive, MemoryStick, Square } from "lucide-react";
import {
  Menubar,
  MenubarContent,
  MenubarMenu,
  MenubarRadioGroup,
  MenubarRadioItem,
  MenubarSeparator,
  MenubarTrigger,
} from "@/components/ui/menubar";
import { METRIC_LABELS, RANGE_OPTIONS, type MetricView, type RangeKey } from "./constants";

function MetricIcon({ view, className }: { view: MetricView; className?: string }) {
  if (view === "cpu") return <Cpu className={className} />;
  if (view === "memory") return <MemoryStick className={className} />;
  return <HardDrive className={className} />;
}

/** The Track page's top bar: process/app identity on the left, and every
 * control (metric view, time range, export, stop) combined into one
 * Menubar on the right — matching the shadcn Menubar the main app Navbar
 * uses for File/Edit/View. */
export function TrackHeader({
  name,
  pids,
  view,
  onViewChange,
  range,
  onRangeChange,
  onExport,
  exportDisabled,
  showStop,
  onStop,
}: {
  name: string;
  pids: number[];
  view: MetricView;
  onViewChange: (view: MetricView) => void;
  range: RangeKey;
  onRangeChange: (range: RangeKey) => void;
  onExport: () => void;
  exportDisabled: boolean;
  showStop: boolean;
  onStop: () => void;
}) {
  return (
    <div className="flex shrink-0 flex-wrap items-center justify-between gap-2 border-b border-teal px-4 py-3">
      <div className="flex items-center gap-2">
        <h2 className="text-sm font-semibold text-navy">{name}</h2>
        <span className="text-xs text-muted-foreground">
          {pids.length > 1 ? `${pids.length} processes` : `PID ${pids[0]}`}
        </span>
      </div>

      <Menubar className="border-teal bg-background">
        <MenubarMenu>
          <MenubarTrigger className="gap-1.5 text-navy">
            <MetricIcon view={view} className="h-3.5 w-3.5" />
            {METRIC_LABELS[view]}
          </MenubarTrigger>
          <MenubarContent align="start">
            <MenubarRadioGroup value={view} onValueChange={(value) => onViewChange(value as MetricView)}>
              <MenubarRadioItem value="cpu">
                <Cpu className="h-3.5 w-3.5" />
                CPU
              </MenubarRadioItem>
              <MenubarRadioItem value="memory">
                <MemoryStick className="h-3.5 w-3.5" />
                Memory
              </MenubarRadioItem>
              <MenubarRadioItem value="storage">
                <HardDrive className="h-3.5 w-3.5" />
                Storage
              </MenubarRadioItem>
            </MenubarRadioGroup>
          </MenubarContent>
        </MenubarMenu>

        <MenubarMenu>
          <MenubarTrigger className="gap-1.5 text-navy">{RANGE_OPTIONS[range].label}</MenubarTrigger>
          <MenubarContent align="start">
            <MenubarRadioGroup value={range} onValueChange={(value) => onRangeChange(value as RangeKey)}>
              {(Object.keys(RANGE_OPTIONS) as RangeKey[]).map((key) => (
                <MenubarRadioItem key={key} value={key}>
                  {RANGE_OPTIONS[key].label}
                </MenubarRadioItem>
              ))}
            </MenubarRadioGroup>
          </MenubarContent>
        </MenubarMenu>

        <MenubarSeparator className="mx-0.5 h-4 w-px" />

        <button
          type="button"
          onClick={onExport}
          disabled={exportDisabled}
          className="flex items-center gap-1.5 rounded-sm px-1.5 py-0.5 text-sm font-medium text-navy outline-hidden select-none hover:bg-muted disabled:pointer-events-none disabled:opacity-50"
        >
          <Download className="h-3.5 w-3.5" />
          Export CSV
        </button>
        {showStop && (
          <button
            type="button"
            onClick={onStop}
            className="flex items-center gap-1.5 rounded-sm px-1.5 py-0.5 text-sm font-medium text-destructive outline-hidden select-none hover:bg-destructive/10"
          >
            <Square className="h-3.5 w-3.5" />
            Stop tracking
          </button>
        )}
      </Menubar>
    </div>
  );
}
