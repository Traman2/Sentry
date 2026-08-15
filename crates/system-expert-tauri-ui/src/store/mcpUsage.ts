import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

/** Mirrors `crates/system-expert-core/src/mcp_usage/models.rs::McpClient`. */
export interface McpClient {
  id: number;
  key: string;
  display_name: string;
  /** `"claude-code" | "claude-desktop" | "python-agent" | "chatgpt" | "grok" | "unknown"` —
   * best-effort, never blocks a client from being recorded. */
  kind: string;
  protocol_name: string | null;
  protocol_version: string | null;
  last_pid: number | null;
  last_process_name: string | null;
  call_count: number;
  first_seen_ms: number;
  last_seen_ms: number;
}

/** Mirrors `crates/system-expert-core/src/mcp_usage/models.rs::McpToolCall`. */
export interface McpToolCall {
  id: number;
  client_id: number;
  tool_name: string;
  params_json: string | null;
  pid: number | null;
  process_name: string | null;
  /** `"ok"` — the call reached and ran a tool (whether or not the tool's own result was a
   * tool-level error the caller sees) — or `"protocol_error"`, a call that never reached a
   * tool body (unknown tool name, malformed request). */
  status: "ok" | "protocol_error";
  duration_ms: number;
  error_message: string | null;
  created_at_ms: number;
}

/** Mirrors `crates/system-expert-core/src/mcp_usage/models.rs::McpClientDetail`. */
export interface McpClientDetail extends McpClient {
  /** Most recent first. */
  calls: McpToolCall[];
}

interface McpUsageStore {
  /** All known clients, most recently active first — the sidebar list. */
  clients: McpClient[];
  loaded: boolean;
  refresh: () => Promise<void>;
  getClientUsage: (id: number) => Promise<McpClientDetail | null>;
  deleteClient: (id: number) => Promise<void>;
}

export const useMcpUsageStore = create<McpUsageStore>((set) => ({
  clients: [],
  loaded: false,

  refresh: async () => {
    const clients = await invoke<McpClient[]>("list_mcp_clients");
    set({ clients, loaded: true });
  },

  getClientUsage: async (id) => {
    return invoke<McpClientDetail | null>("get_mcp_client_usage", { id });
  },

  deleteClient: async (id) => {
    await invoke<boolean>("delete_mcp_client", { id });
    set((state) => ({ clients: state.clients.filter((c) => c.id !== id) }));
  },
}));
