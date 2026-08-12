import { invoke } from "@tauri-apps/api/core";
import { ArrowDown, TriangleAlert } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { MessageGroup } from "@/components/ui/message";
import { useChatStore, type ChatSpaceDetail } from "@/store/chat";
import { useTabStore } from "@/store/tabs";
import { ChatEmptyState, ChatMissingState } from "./ChatSpace/ChatEmptyState";
import { ChatHeader } from "./ChatSpace/ChatHeader";
import { Composer } from "./ChatSpace/Composer";
import {
  AssistantMessage,
  MessageListSkeleton,
  PendingAssistantMessage,
  UserMessage,
} from "./ChatSpace/MessageItem";
import type { ModelId } from "./ChatSpace/constants";

/** How close to the bottom (in px) still counts as "pinned to the bottom" —
 * below this the transcript stops auto-following new messages and offers a
 * jump-to-latest button instead. */
const BOTTOM_THRESHOLD_PX = 32;

function ChatSpace({ tabId }: { tabId: string }) {
  const chatSpaceId = Number(tabId);
  const [detail, setDetail] = useState<ChatSpaceDetail | null>(null);
  const [notFound, setNotFound] = useState(false);
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [model, setModel] = useState<ModelId>("qwen");
  // The message being sent, rendered optimistically so the user's own turn
  // appears the instant they hit send rather than after the round trip.
  const [pending, setPending] = useState<string | null>(null);
  const [atBottom, setAtBottom] = useState(true);

  const sendMessage = useChatStore((state) => state.sendMessage);
  const renameTab = useTabStore((state) => state.renameTab);
  const scrollRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    let cancelled = false;
    invoke<ChatSpaceDetail | null>("get_chat_space", { id: chatSpaceId }).then((next) => {
      if (cancelled) return;
      setDetail(next);
      setNotFound(next === null);
    });
    return () => {
      cancelled = true;
    };
  }, [chatSpaceId]);

  const updateAtBottom = () => {
    const el = scrollRef.current;
    if (!el) return;
    setAtBottom(el.scrollHeight - el.scrollTop - el.clientHeight < BOTTOM_THRESHOLD_PX);
  };

  // Follow the conversation only while the user is already reading the bottom;
  // a send always wins, since they just asked for the reply.
  useEffect(() => {
    if (!atBottom && pending === null) return;
    bottomRef.current?.scrollIntoView({ block: "end" });
  }, [detail?.messages.length, pending, atBottom]);

  // Scroll events alone can't keep `atBottom` honest: the transcript also
  // changes height when the pane resizes or the web font swaps in, and a
  // reflow that removes the overflow fires no scroll event — leaving a
  // jump-to-latest button stranded over content that already fits.
  useEffect(() => {
    const el = scrollRef.current;
    const content = el?.firstElementChild;
    if (!el || !content) return;
    const observer = new ResizeObserver(() => updateAtBottom());
    observer.observe(el);
    observer.observe(content);
    return () => observer.disconnect();
  }, []);

  const scrollToBottom = () => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth", block: "end" });
  };

  const handleSend = async (text?: string) => {
    const content = (text ?? draft).trim();
    if (!content || sending || notFound) return;
    setDraft("");
    setError(null);
    setPending(content);
    setSending(true);
    try {
      const updated = await sendMessage(chatSpaceId, content);
      setDetail(updated);
      renameTab(tabId, updated.title);
    } catch (e) {
      // Put the text back in the composer — silently swallowing it would lose
      // whatever the user just typed.
      setDraft(content);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSending(false);
      setPending(null);
    }
  };

  const messages = detail?.messages ?? [];
  const isEmpty = detail !== null && messages.length === 0 && pending === null;

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm">
      <ChatHeader title={detail?.title ?? "New Chat"} messageCount={messages.length} />

      <div className="relative min-h-0 flex-1">
        <div
          ref={scrollRef}
          onScroll={updateAtBottom}
          className="h-full overflow-y-auto"
        >
          {/* `min-h-full` lets the placeholder states — which are `flex-1` —
              centre themselves in the pane instead of hugging the top. */}
          <div className="mx-auto flex min-h-full w-full max-w-3xl flex-col px-6 py-6">
            {notFound ? (
              <ChatMissingState />
            ) : detail === null ? (
              <MessageListSkeleton />
            ) : isEmpty ? (
              <ChatEmptyState onPick={(prompt) => handleSend(prompt)} />
            ) : (
              <MessageGroup className="gap-7">
                {messages.map((message) =>
                  message.role === "user" ? (
                    <UserMessage key={message.id} message={message} />
                  ) : (
                    <AssistantMessage key={message.id} message={message} />
                  ),
                )}
                {pending !== null && (
                  <>
                    <UserMessage
                      message={{
                        id: -1,
                        chat_space_id: chatSpaceId,
                        role: "user",
                        content: pending,
                        created_at_ms: Date.now(),
                      }}
                    />
                    <PendingAssistantMessage />
                  </>
                )}
              </MessageGroup>
            )}
            <div ref={bottomRef} />
          </div>
        </div>

        {messages.length > 0 && !atBottom && (
          <Button
            variant="outline"
            size="icon-sm"
            aria-label="Jump to latest message"
            onClick={scrollToBottom}
            className="absolute bottom-3 left-1/2 -translate-x-1/2 rounded-full border-teal/50 shadow-sm hover:bg-teal/25"
          >
            <ArrowDown className="size-4" />
          </Button>
        )}
      </div>

      {error && (
        <div className="shrink-0 px-4">
          <div className="mx-auto flex w-full max-w-3xl items-center gap-2 rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2 text-[11px] text-destructive">
            <TriangleAlert className="size-3.5 shrink-0" />
            <span className="min-w-0 flex-1">Couldn’t send that message. {error}</span>
          </div>
        </div>
      )}

      <Composer
        draft={draft}
        onDraftChange={setDraft}
        onSend={() => handleSend()}
        model={model}
        onModelChange={setModel}
        disabled={notFound}
        sending={sending}
        placeholder={
          notFound ? "This chat no longer exists" : "Ask about CPU, memory, processes…"
        }
      />
    </div>
  );
}

export default ChatSpace;
