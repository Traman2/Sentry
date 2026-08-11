import { Archive, Radar, Trash2 } from "lucide-react";
import { useEffect, type MouseEvent } from "react";
import type { TrackedProcess } from "@/store/tracking";
import { useTrackingStore } from "@/store/tracking";
import { useTabStore } from "@/store/tabs";

function openTrackTab(
  openTab: (tab: { id: string; type: "track"; title: string }) => void,
  tracked: TrackedProcess,
) {
  openTab({ id: `track-${tracked.id}`, type: "track", title: `Track: ${tracked.name}` });
}

function DetailsView() {
  const trackedProcesses = useTrackingStore((state) => state.trackedProcesses);
  const loaded = useTrackingStore((state) => state.loaded);
  const refresh = useTrackingStore((state) => state.refresh);
  const deleteTracked = useTrackingStore((state) => state.deleteTracked);
  const openTab = useTabStore((state) => state.openTab);
  const discardTab = useTabStore((state) => state.discardTab);

  useEffect(() => {
    if (!loaded) refresh();
  }, [loaded, refresh]);

  const handleDelete = async (e: MouseEvent, tracked: TrackedProcess) => {
    e.stopPropagation();
    await deleteTracked(tracked.id);
    discardTab(`track-${tracked.id}`);
  };

  return (
    <div className="flex h-full flex-col gap-1">
      <p className="text-[10px] font-semibold tracking-wide text-navy/60 uppercase">
        Recently tracked
      </p>
      <div className="flex flex-col gap-0.5 overflow-y-auto">
        {trackedProcesses.length === 0 && (
          <p className="text-xs text-muted-foreground">
            Nothing tracked yet. Click "Track" on a process in the Resource Monitor.
          </p>
        )}
        {trackedProcesses.map((tracked) => (
          <div
            key={tracked.id}
            role="button"
            tabIndex={0}
            onClick={() => openTrackTab(openTab, tracked)}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") openTrackTab(openTab, tracked);
            }}
            className="group flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-navy/80 hover:bg-teal/25"
          >
            {tracked.status === "active" ? (
              <Radar className="h-3.5 w-3.5 shrink-0 animate-pulse text-emerald-600" />
            ) : (
              <Archive className="h-3.5 w-3.5 shrink-0 text-navy/40" />
            )}
            <div className="min-w-0 flex-1">
              <span className="block truncate text-xs">{tracked.name}</span>
              <span className="block truncate text-[10px] text-muted-foreground">
                {tracked.pids.length > 1 ? `${tracked.pids.length} processes` : `PID ${tracked.pids[0]}`}
                {" · "}
                {tracked.status === "active" ? "Live" : "Ended"}
              </span>
            </div>
            <button
              type="button"
              aria-label={`Delete tracked session for "${tracked.name}"`}
              onClick={(e) => handleDelete(e, tracked)}
              className="shrink-0 rounded-md p-1 text-navy/40 opacity-0 hover:bg-teal/40 hover:text-navy group-hover:opacity-100"
            >
              <Trash2 className="h-3 w-3" />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

export default DetailsView;
