import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ArrowDown, TriangleAlert } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import { MessageGroup } from "@/components/ui/message";
import { useAgentStore } from "@/store/agent";
import { useChatStore, type ChatSpaceDetail } from "@/store/chat";
import { useTabStore } from "@/store/tabs";
import { ChatEmptyState, ChatMissingState } from "./ChatSpace/ChatEmptyState";
import { ChatHeader } from "./ChatSpace/ChatHeader";
import { Composer } from "./ChatSpace/Composer";
import {
  AssistantMessage,
  ErrorMessage,
  InterruptedMessage,
  MessageListSkeleton,
  PendingAssistantMessage,
  UserMessage,
} from "./ChatSpace/MessageItem";

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
  // The message being sent, rendered optimistically so the user's own turn
  // appears the instant they hit send rather than after the round trip.
  const [pending, setPending] = useState<string | null>(null);
  const [atBottom, setAtBottom] = useState(true);
  /** Live progress from the agent for the turn in flight. */
  const [steps, setSteps] = useState<string[]>([]);

  const sendMessage = useChatStore((state) => state.sendMessage);
  const model = useAgentStore((state) => state.model);
  const setModel = useAgentStore((state) => state.setModel);
  const agentStatus = useAgentStore((state) => state.status);
  const renameTab = useTabStore((state) => state.renameTab);
  const scrollRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  const refresh = useCallback(async () => {
    const next = await invoke<ChatSpaceDetail | null>("get_chat_space", { id: chatSpaceId });
    setDetail(next);
    setNotFound(next === null);
    return next;
  }, [chatSpaceId]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const messages = detail?.messages ?? [];
  // `send_chat_message` records only the user's turn — the agent writes the reply
  // separately — so a transcript ending on a user message is one still being answered.
  // Deriving this rather than holding a flag means it survives a tab switch or remount.
  const awaitingReply = messages.length > 0 && messages[messages.length - 1].role === "user";

  // The agent posts its reply straight into the database from another process, so the
  // backend emits an event when that happens rather than the UI guessing.
  useEffect(() => {
    const unlisten = listen<number>("mcp://chat-updated", (event) => {
      if (event.payload === chatSpaceId) void refresh();
    });
    return () => {
      void unlisten.then((off) => off());
    };
  }, [chatSpaceId, refresh]);

  // Progress from the agent as it works. These are never persisted — they exist only for
  // the wait, and the arrival of a real message is what retires them (see below).
  useEffect(() => {
    const unlisten = listen<{ chat_space_id: number; step: string }>(
      "mcp://thinking",
      (event) => {
        if (event.payload.chat_space_id !== chatSpaceId) return;
        setSteps((current) => [...current, event.payload.step]);
      },
    );
    return () => {
      void unlisten.then((off) => off());
    };
  }, [chatSpaceId]);

  // Clearing on message count rather than in the send/stop handlers means every way a turn
  // can end — a reply, an error, an interrupt — retires the steps through one path.
  useEffect(() => {
    setSteps([]);
  }, [messages.length]);


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
  }, [messages.length, pending, atBottom]);

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

  const isEmpty = detail !== null && messages.length === 0 && pending === null;
  const showPendingReply = pending !== null || awaitingReply;

  const handleStop = async () => {
    // The backend closes the turn out in the transcript and tells the agent to drop it, so
    // this unblocks even when the agent is the thing that's wedged.
    const updated = await invoke<ChatSpaceDetail | null>("interrupt_chat", {
      chatSpaceId: chatSpaceId,
    });
    // `null` means a reply landed first and the interrupt was a no-op — refresh to pick it
    // up rather than leaving the transcript a beat behind.
    if (updated) setDetail(updated);
    else void refresh();
  };

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
                {messages.map((message, index) => {
                  if (message.role === "user")
                    return <UserMessage key={message.id} message={message} />;
                  if (message.role === "error")
                    return <ErrorMessage key={message.id} message={message} />;
                  if (message.role === "interrupted")
                    return <InterruptedMessage key={message.id} message={message} />;
                  // A reply always immediately follows the user turn it answers, so the
                  // previous row is the timer's start — see AssistantMessage.
                  const previous = messages[index - 1];
                  return (
                    <AssistantMessage
                      key={message.id}
                      message={message}
                      respondedToMessage={previous?.role === "user" ? previous : undefined}
                    />
                  );
                })}
                {pending !== null && (
                  <UserMessage
                    message={{
                      id: -1,
                      chat_space_id: chatSpaceId,
                      role: "user",
                      content: pending,
                      details: null,
                      created_at_ms: Date.now(),
                    }}
                  />
                )}
                {showPendingReply && (
                  <PendingAssistantMessage
                    agentDown={agentStatus !== null && !agentStatus.running}
                    detail={agentStatus?.detail}
                    onStop={awaitingReply ? handleStop : undefined}
                    steps={steps}
                  />
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
