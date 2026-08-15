import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { ProcessSamplePoint, SystemSamplePoint } from "./types";

/**
 * System-wide CPU/mem/network history for the last `sinceSecs` seconds, backed by
 * the SQLite history recorded independently of this hook (see
 * `system_expert_core::spawn_recorder`). Refetches on an interval so a chart using this
 * stays live.
 */
export function useSystemTimeline(sinceSecs: number, refetchIntervalMs = 5000) {
  const [points, setPoints] = useState<SystemSamplePoint[]>([]);

  useEffect(() => {
    let cancelled = false;

    async function refresh() {
      const next = await invoke<SystemSamplePoint[]>("get_system_timeline", { sinceSecs });
      if (!cancelled) setPoints(next);
    }

    refresh();
    const interval = setInterval(refresh, refetchIntervalMs);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [sinceSecs, refetchIntervalMs]);

  return points;
}

/**
 * CPU/mem/disk-I/O history for a fixed set of `pids` over the last `sinceSecs`
 * seconds — backs the Track page for both single processes and apps (an app
 * being tracked is a fixed set of pids captured at track-start). Pass
 * `enabled: false` to stop polling (e.g. once tracking has ended) without
 * unmounting the hook.
 */
export function useAppTimeline(
  pids: number[],
  sinceSecs: number,
  enabled: boolean,
  refetchIntervalMs = 4000,
) {
  const [points, setPoints] = useState<ProcessSamplePoint[]>([]);
  const pidsKey = pids.join(",");

  useEffect(() => {
    if (!enabled || pids.length === 0) return;

    let cancelled = false;

    async function refresh() {
      const next = await invoke<ProcessSamplePoint[]>("get_app_timeline", {
        pids,
        sinceSecs,
      });
      if (!cancelled) setPoints(next);
    }

    refresh();
    const interval = setInterval(refresh, refetchIntervalMs);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
    // Deliberately keyed on pidsKey (a string), not pids itself — pids is a fixed
    // set for a given track session, so keying on the array's referential
    // identity would refetch every render for no reason.
  }, [pidsKey, sinceSecs, enabled, refetchIntervalMs]);

  return points;
}

/**
 * CPU/mem/disk-I/O history for a single process over the last `sinceSecs` seconds.
 * Pass `pid: null` when no process is selected for tracking — the hook then skips
 * fetching and returns an empty array.
 */
export function useProcessTimeline(
  pid: number | null,
  sinceSecs: number,
  refetchIntervalMs = 5000,
) {
  const [points, setPoints] = useState<ProcessSamplePoint[]>([]);

  useEffect(() => {
    if (pid === null) {
      setPoints([]);
      return;
    }

    let cancelled = false;

    async function refresh() {
      const next = await invoke<ProcessSamplePoint[]>("get_process_timeline", {
        pid,
        sinceSecs,
      });
      if (!cancelled) setPoints(next);
    }

    refresh();
    const interval = setInterval(refresh, refetchIntervalMs);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [pid, sinceSecs, refetchIntervalMs]);

  return points;
}
