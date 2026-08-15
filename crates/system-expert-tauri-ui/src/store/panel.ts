import { create } from "zustand";
import type { PanelId } from "../config/panels";

interface PanelStore {
  activePanel: PanelId | null;
  togglePanel: (panel: PanelId) => void;
}

export const usePanelStore = create<PanelStore>((set, get) => ({
  activePanel: null,
  togglePanel: (panel) =>
    set({ activePanel: get().activePanel === panel ? null : panel }),
}));