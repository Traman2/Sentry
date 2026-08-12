import { Cpu, HardDrive, MemoryStick } from "lucide-react";
import { TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { RANGE_OPTIONS, type RangeKey } from "./constants";

const RANGE_KEYS = Object.keys(RANGE_OPTIONS) as RangeKey[];

/** The Track page's view controls: which metric is charted (tabs, left) and how
 * far back the window reaches (segmented control, right). Must render inside a
 * `Tabs` root — `TabsList` reads that context for the active view. */
export function MetricToolbar({
  range,
  onRangeChange,
}: {
  range: RangeKey;
  onRangeChange: (range: RangeKey) => void;
}) {
  return (
    <div className="flex shrink-0 flex-wrap items-center justify-between gap-2 border-b border-teal/50 px-4 py-2">
      <TabsList>
        <TabsTrigger value="cpu" className="px-3 data-active:text-navy">
          <Cpu />
          CPU
        </TabsTrigger>
        <TabsTrigger value="memory" className="px-3 data-active:text-navy">
          <MemoryStick />
          Memory
        </TabsTrigger>
        <TabsTrigger value="storage" className="px-3 data-active:text-navy">
          <HardDrive />
          Storage
        </TabsTrigger>
      </TabsList>

      <ToggleGroup
        variant="outline"
        size="sm"
        spacing={0}
        value={[range]}
        onValueChange={(value) => {
          // Clicking the already-active range would otherwise clear the group
          // and leave the page with no window selected.
          const next = value[0] as RangeKey | undefined;
          if (next) onRangeChange(next);
        }}
      >
        {RANGE_KEYS.map((key) => (
          <ToggleGroupItem
            key={key}
            value={key}
            aria-label={`Show the last ${RANGE_OPTIONS[key].label}`}
            className="border-teal text-navy/70 hover:bg-teal/20 hover:text-navy aria-pressed:bg-teal/40 aria-pressed:text-navy"
          >
            {RANGE_OPTIONS[key].label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
    </div>
  );
}
