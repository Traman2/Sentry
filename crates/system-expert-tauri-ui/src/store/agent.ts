import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { create } from "zustand";
import type { ModelId } from "../tab-pages/ChatSpace/constants";

/** Mirrors `crates/system-expert-tauri-ui/src-tauri/src/agent.rs::AgentStatus`. */
export interface AgentStatus {
  running: boolean;
  pid: number | null;
  model: string;
  /** Human-readable state — worth surfacing when `running` is false. */
  detail: string;
}

interface AgentStore {
  status: AgentStatus | null;
  /** The model shown in the composer's picker. */
  model: ModelId;
  refresh: () => Promise<void>;
  /** Records the model choice with the backend.
   *
   * The running agent picks this up over MCP on its next poll and swaps in place, so
   * switching models mid-conversation keeps its history — nothing restarts. */
  setModel: (model: ModelId) => Promise<void>;
  start: () => Promise<void>;
  stop: () => Promise<void>;
}

export const useAgentStore = create<AgentStore>((set) => ({
  status: null,
  model: "qwen",

  refresh: async () => {
    const status = await invoke<AgentStatus>("get_agent_status");
    set({ status });
  },

  setModel: async (model) => {
    // Optimistic: the picker should feel instant, and the backend call only records a
    // string — there is no failure mode worth blocking the UI on.
    set({ model });
    const status = await invoke<AgentStatus>("set_agent_model", { model });
    set({ status });
  },

  start: async () => {
    const status = await invoke<AgentStatus>("start_agent");
    set({ status });
  },

  stop: async () => {
    const status = await invoke<AgentStatus>("stop_agent");
    set({ status });
  },
}));

/** Keeps `status` fresh without polling.
 *
 * The agent holds a WebSocket to the backend for as long as it's alive, so the backend
 * emits on connect and disconnect — covering a clean exit, a crash, and a kill alike. The
 * one-shot fetch is for the case where the agent connected before this listener attached. */
export function watchAgentStatus(): () => void {
  const { refresh } = useAgentStore.getState();
  refresh().catch(() => {});

  const unlisten = listen<boolean>("agent://connection", () => {
    refresh().catch(() => {});
  });

  return () => {
    void unlisten.then((off) => off());
  };
}
