import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import type { TrackedArchive, TrackedProcess } from "../tab-pages/ResourceMonitor/types";

export type { TrackedArchive, TrackedProcess };

interface TrackingStore {
  /** All tracked sessions (active and ended), most recently started first —
   * backs the Details View sidebar's "recently tracked" list. */
  trackedProcesses: TrackedProcess[];
  loaded: boolean;
  refresh: () => Promise<void>;
  /** Starts a new tracking session over a fixed set of pids and adds it to the
   * store. `pids` is a single-element array for a lone process, or every pid
   * backing an app. */
  startTracking: (name: string, pids: number[]) => Promise<TrackedProcess>;
  /** Ends a session — manually, or because the caller detected every tracked pid
   * is gone — archiving its samples on the backend. Safe to call more than once. */
  endTracking: (id: number) => Promise<TrackedProcess>;
  /** Deletes a tracked session (and its archive file, if any) from the store. */
  deleteTracked: (id: number) => Promise<void>;
  /** Fetches the archived samples for an ended session, e.g. once its raw
   * history rows have aged out of the live SQLite retention window. */
  getArchive: (id: number) => Promise<TrackedArchive | null>;
}

export const useTrackingStore = create<TrackingStore>((set, get) => ({
  trackedProcesses: [],
  loaded: false,

  refresh: async () => {
    const trackedProcesses = await invoke<TrackedProcess[]>("list_tracked_processes", {
      name: null,
    });
    set({ trackedProcesses, loaded: true });
  },

  startTracking: async (name, pids) => {
    const tracked = await invoke<TrackedProcess>("start_tracking", { name, pids });
    set((state) => ({ trackedProcesses: [tracked, ...state.trackedProcesses] }));
    return tracked;
  },

  endTracking: async (id) => {
    const ended = await invoke<TrackedProcess>("end_tracking", { id });
    set((state) => ({
      trackedProcesses: state.trackedProcesses.map((t) => (t.id === id ? ended : t)),
    }));
    return ended;
  },

  deleteTracked: async (id) => {
    await invoke<boolean>("delete_tracked_process", { id });
    set((state) => ({
      trackedProcesses: state.trackedProcesses.filter((t) => t.id !== id),
    }));
  },

  getArchive: async (id) => {
    const existing = get().trackedProcesses.find((t) => t.id === id);
    if (!existing?.archive_path) return null;
    return invoke<TrackedArchive | null>("get_tracked_archive", { id });
  },
}));
