import { create } from "zustand";
import type { TabType } from "../config/tabs";

export type { TabType };

export interface Tab {
  id: string;
  type: TabType;
  title: string;
}

interface TabStore {
  tabs: Tab[];
  activeTabId: string | null;
  openTab: (tab: Tab) => void;
  closeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
}

const defaultTab: Tab = { id: "welcome", type: "welcome", title: "Welcome" };

export const useTabStore = create<TabStore>((set, get) => ({
  tabs: [defaultTab],
  activeTabId: defaultTab.id,

  openTab: (tab) => {
    const exists = get().tabs.some((t) => t.id === tab.id);
    if (exists) {
      set({ activeTabId: tab.id });
      return;
    }
    set((state) => ({ tabs: [...state.tabs, tab], activeTabId: tab.id }));
  },

  closeTab: (id) => {
    const { tabs, activeTabId } = get();
    const remaining = tabs.filter((t) => t.id !== id);
    const newActiveTabId =
      activeTabId === id
        ? remaining.length > 0
          ? remaining[remaining.length - 1].id
          : null
        : activeTabId;
    set({ tabs: remaining, activeTabId: newActiveTabId });
  },

  setActiveTab: (id) => set({ activeTabId: id }),
}));
