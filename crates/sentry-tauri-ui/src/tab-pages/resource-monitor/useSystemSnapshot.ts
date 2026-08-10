import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { SystemSnapshot } from "./types";

const EMPTY_SNAPSHOT: SystemSnapshot = {
  timestamp_ms: 0,
  processes: [],
  networks: [],
};

export function useSystemSnapshot(intervalMs = 2000) {
  const [snapshot, setSnapshot] = useState<SystemSnapshot>(EMPTY_SNAPSHOT);

  useEffect(() => {
    let cancelled = false;

    async function refresh() {
      const next = await invoke<SystemSnapshot>("get_snapshot");
      if (!cancelled) setSnapshot(next);
    }

    refresh();
    const interval = setInterval(refresh, intervalMs);
    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [intervalMs]);

  return snapshot;
}
