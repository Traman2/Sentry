import { Radar } from "lucide-react";
import { useEffect, useMemo } from "react";
import { PanelEmpty, PanelListItem, PanelSection, StatusDot } from "@/components/PanelList";
import type { TrackedProcess } from "@/store/tracking";
import { useTrackingStore } from "@/store/tracking";
import { useTabStore } from "@/store/tabs";

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

  // Live sessions are what you come back to; ended ones are history. Splitting
  // them carries the status without needing a second line on every row.
  const { active, ended } = useMemo(
    () => ({
      active: trackedProcesses.filter((tracked) => tracked.status === "active"),
      ended: trackedProcesses.filter((tracked) => tracked.status !== "active"),
    }),
    [trackedProcesses],
  );

  const handleDelete = async (tracked: TrackedProcess) => {
    await deleteTracked(tracked.id);
    discardTab(`track-${tracked.id}`);
  };

  const renderRow = (tracked: TrackedProcess) => (
    <PanelListItem
      key={tracked.id}
      leading={<StatusDot live={tracked.status === "active"} />}
      title={tracked.name}
      onOpen={() =>
        openTab({
          id: `track-${tracked.id}`,
          type: "track",
          title: `Track: ${tracked.name}`,
        })
      }
      onDelete={() => handleDelete(tracked)}
      deleteLabel={`Delete tracked session for "${tracked.name}"`}
    />
  );

  if (trackedProcesses.length === 0) {
    return (
      <PanelEmpty
        icon={<Radar />}
        title="Nothing tracked yet"
        description={'Hit "Track" on a process in the Resource Monitor to start a session.'}
      />
    );
  }

  return (
    <div className="flex flex-col gap-3">
      {active.length > 0 && (
        <PanelSection label="Active">{active.map(renderRow)}</PanelSection>
      )}
      {ended.length > 0 && <PanelSection label="Ended">{ended.map(renderRow)}</PanelSection>}
    </div>
  );
}

export default DetailsView;
