import { create } from "zustand";
import type { TabType } from "../config/tabs";

export type { TabType };

export interface Tab {
  id: string;
  type: TabType;
  title: string;
}

const MAX_RECENT = 10;

interface TabStore {
  tabs: Tab[];
  activeTabId: string | null;
  /** Up to MAX_RECENT most recently opened pages, most recent first — Welcome's
   * "Recent" panel links back into these. Excludes the Welcome tab itself. Survives
   * a tab being closed (unlike `tabs`), but is in-memory only and resets on relaunch. */
  recent: Tab[];
  openTab: (tab: Tab) => void;
  closeTab: (id: string) => void;
  /** Closes the tab (if open) and, unlike `closeTab`, also drops it from `recent` —
   * use this when the underlying resource itself was deleted, so Welcome's "Recent"
   * panel doesn't keep offering a dead link back to it. */
  discardTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  renameTab: (id: string, title: string) => void;
}

const defaultTab: Tab = { id: "welcome", type: "welcome", title: "Welcome" };

export const useTabStore = create<TabStore>((set) => ({
  tabs: [defaultTab],
  activeTabId: defaultTab.id,
  recent: [],

  openTab: (tab) =>
    set((state) => {
      const tabs = state.tabs.some((t) => t.id === tab.id)
        ? state.tabs
        : [...state.tabs, tab];

      const recent =
        tab.type === "welcome"
          ? state.recent
          : [tab, ...state.recent.filter((r) => r.id !== tab.id)].slice(0, MAX_RECENT);

      return { tabs, activeTabId: tab.id, recent };
    }),

  closeTab: (id) =>
    set((state) => {
      const remaining = state.tabs.filter((t) => t.id !== id);
      const activeTabId =
        state.activeTabId === id
          ? remaining.length > 0
            ? remaining[remaining.length - 1].id
            : null
          : state.activeTabId;
      return { tabs: remaining, activeTabId };
    }),

  discardTab: (id) =>
    set((state) => {
      const remaining = state.tabs.filter((t) => t.id !== id);
      const activeTabId =
        state.activeTabId === id
          ? remaining.length > 0
            ? remaining[remaining.length - 1].id
            : null
          : state.activeTabId;
      return {
        tabs: remaining,
        activeTabId,
        recent: state.recent.filter((t) => t.id !== id),
      };
    }),

  setActiveTab: (id) => set({ activeTabId: id }),

  renameTab: (id, title) =>
    set((state) => ({
      tabs: state.tabs.map((t) => (t.id === id ? { ...t, title } : t)),
      recent: state.recent.map((t) => (t.id === id ? { ...t, title } : t)),
    })),
}));
