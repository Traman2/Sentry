import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useRef, useState } from "react";
import { useTrackingStore } from "@/store/tracking";
import type { ProcessSamplePoint, TrackedArchive, TrackedProcess } from "../ResourceMonitor/types";
import { useAppTimeline } from "../ResourceMonitor/useHistoryTimeline";
import { useSystemSnapshot } from "../ResourceMonitor/useSystemSnapshot";
import { DEATH_CONFIRMATION_POLLS, RANGE_OPTIONS, type RangeKey } from "./constants";
import { filterToRange } from "./series";

/**
 * All the state and side effects behind the Track page: loading the tracked
 * session, polling its live samples (or, once ended, its frozen archive),
 * and detecting when every tracked pid has disappeared so tracking can be
 * halted automatically.
 */
export function useTrackedSession(trackedId: number, range: RangeKey) {
  const [tracked, setTracked] = useState<TrackedProcess | null>(null);
  const [notFound, setNotFound] = useState(false);
  const [archive, setArchive] = useState<TrackedArchive | null>(null);
  const [endReason, setEndReason] = useState<"died" | "manual" | null>(null);
  const missCountRef = useRef(0);
  const endTracking = useTrackingStore((state) => state.endTracking);
  const storeEntry = useTrackingStore((state) =>
    state.trackedProcesses.find((t) => t.id === trackedId),
  );
  const snapshot = useSystemSnapshot(2000);

  useEffect(() => {
    let cancelled = false;
    invoke<TrackedProcess | null>("get_tracked_process", { id: trackedId }).then((next) => {
      if (cancelled) return;
      setTracked(next);
      setNotFound(next === null);
    });
    return () => {
      cancelled = true;
    };
  }, [trackedId]);

  useEffect(() => {
    if (storeEntry) setTracked(storeEntry);
  }, [storeEntry]);

  useEffect(() => {
    if (!tracked || tracked.status !== "active" || snapshot.timestamp_ms === 0) return;

    const alive = snapshot.processes.some((p) => tracked.pids.includes(p.pid));
    if (alive) {
      missCountRef.current = 0;
      return;
    }

    missCountRef.current += 1;
    if (missCountRef.current >= DEATH_CONFIRMATION_POLLS) {
      setEndReason("died");
      endTracking(tracked.id).then(setTracked);
    }
  }, [snapshot, tracked, endTracking]);

  useEffect(() => {
    if (tracked?.status === "ended" && tracked.archive_path && !archive) {
      invoke<TrackedArchive | null>("get_tracked_archive", { id: tracked.id }).then(setArchive);
    }
  }, [tracked?.status, tracked?.archive_path, tracked?.id, archive]);

  const rangeSecs = RANGE_OPTIONS[range].secs;
  const liveSamples = useAppTimeline(
    tracked?.pids ?? [],
    rangeSecs,
    tracked?.status === "active",
    RANGE_OPTIONS[range].refetchMs,
  );

  // Memoized so this stays referentially stable between renders that don't
  // actually change the underlying data (e.g. the 2s `useSystemSnapshot` tick
  // driving the liveness check above) — without that, every derived value
  // downstream (the bucketed table rows, the chart series, and every
  // tanstack table's internal sort/paginate cache) would recompute from
  // scratch on every single render instead of only when new samples
  // actually arrive. That was cheap-looking per call but, at a few seconds
  // of cadence times three full table recomputations, was what froze the UI.
  const samples = useMemo<ProcessSamplePoint[]>(() => {
    if (!tracked) return [];
    if (tracked.status !== "ended") return liveSamples;
    return archive ? filterToRange(archive.samples, rangeSecs, archive.ended_at_ms) : [];
  }, [tracked, archive, rangeSecs, liveSamples]);

  const stopTracking = async () => {
    if (!tracked || tracked.status !== "active") return;
    setEndReason("manual");
    const ended = await endTracking(tracked.id);
    setTracked(ended);
  };

  return { tracked, notFound, samples, endReason, stopTracking };
}
