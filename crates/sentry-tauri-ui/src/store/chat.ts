import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

/** Mirrors `crates/sentry-core/src/chat/mod.rs::ChatSpace`. */
export interface ChatSpace {
  id: number;
  title: string;
  created_at_ms: number;
  updated_at_ms: number;
}

/** Mirrors `crates/sentry-core/src/chat/mod.rs::ChatMessage`. */
export interface ChatMessage {
  id: number;
  chat_space_id: number;
  role: "user" | "assistant";
  content: string;
  created_at_ms: number;
}

/** Mirrors `crates/sentry-core/src/chat/mod.rs::ChatSpaceDetail`. */
export interface ChatSpaceDetail extends ChatSpace {
  messages: ChatMessage[];
}

interface ChatStore {
  /** All chat spaces, most recently active first — the sidebar list. */
  chatSpaces: ChatSpace[];
  loaded: boolean;
  refresh: () => Promise<void>;
  createChatSpace: () => Promise<ChatSpace>;
  /** Merges a space returned from the backend (e.g. after its title changes on
   * first message) into the list and re-sorts it to the front, matching backend
   * ordering (most recently active first). */
  upsertChatSpace: (space: ChatSpace) => void;
  /** Sends a message on an existing chat space and syncs the result into the store.
   * Returns the refreshed detail (including the title, which may have just changed
   * if this was the space's first message) so the caller can e.g. open its tab. */
  sendMessage: (chatSpaceId: number, content: string) => Promise<ChatSpaceDetail>;
  /** Creates a new chat space and immediately sends `content` as its first message
   * — used by "Ask AI" entry points that jump straight into a pre-filled question
   * rather than going through the empty "+ New chat" flow. */
  createChatSpaceWithMessage: (content: string) => Promise<ChatSpaceDetail>;
}

export const useChatStore = create<ChatStore>((set, get) => ({
  chatSpaces: [],
  loaded: false,

  refresh: async () => {
    const chatSpaces = await invoke<ChatSpace[]>("list_chat_spaces");
    set({ chatSpaces, loaded: true });
  },

  createChatSpace: async () => {
    const space = await invoke<ChatSpace>("create_chat_space");
    set((state) => ({ chatSpaces: [space, ...state.chatSpaces] }));
    return space;
  },

  upsertChatSpace: (space) => {
    set((state) => ({
      chatSpaces: [space, ...state.chatSpaces.filter((c) => c.id !== space.id)],
    }));
  },

  sendMessage: async (chatSpaceId, content) => {
    const detail = await invoke<ChatSpaceDetail>("send_chat_message", {
      chatSpaceId,
      content,
    });
    get().upsertChatSpace({
      id: detail.id,
      title: detail.title,
      created_at_ms: detail.created_at_ms,
      updated_at_ms: detail.updated_at_ms,
    });
    return detail;
  },

  createChatSpaceWithMessage: async (content) => {
    const space = await get().createChatSpace();
    return get().sendMessage(space.id, content);
  },
}));
