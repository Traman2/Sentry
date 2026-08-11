import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { ProcessSamplePoint, SystemSamplePoint } from "./types";

/**
 * System-wide CPU/mem/network history for the last `sinceSecs` seconds, backed by
 * the SQLite history recorded independently of this hook (see
 * `sentry_core::spawn_recorder`). Refetches on an interval so a chart using this
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
