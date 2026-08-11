import { invoke } from "@tauri-apps/api/core";
import { ArrowUp, Bot, Check, ChevronDown, Copy, User } from "lucide-react";
import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useChatStore, type ChatMessage, type ChatSpaceDetail } from "@/store/chat";
import { useTabStore } from "@/store/tabs";

type ModelId = "qwen" | "gpt-oss";

const MODEL_LABELS: Record<ModelId, string> = {
  qwen: "Qwen",
  "gpt-oss": "GPT-OSS",
};

function CopyButton({ content }: { content: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(content);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <button
      type="button"
      onClick={handleCopy}
      className="mt-2 flex items-center gap-1 text-xs text-navy/50 hover:text-navy"
    >
      {copied ? <Check className="h-3 w-3" /> : <Copy className="h-3 w-3" />}
      {copied ? "Copied" : "Copy"}
    </button>
  );
}

function MessageRow({ message }: { message: ChatMessage }) {
  const isUser = message.role === "user";
  return (
    <div className="flex gap-3 py-4 last:border-b-0">
      <div
        className={`flex h-7 w-7 shrink-0 items-center justify-center rounded-full ${
          isUser ? "bg-navy text-canvas" : "bg-teal/30 text-navy"
        }`}
      >
        {isUser ? <User className="h-4 w-4" /> : <Bot className="h-4 w-4" />}
      </div>
      <div className="min-w-0 flex-1">
        <p className="text-sm whitespace-pre-wrap text-navy">{message.content}</p>
        {!isUser && <CopyButton content={message.content} />}
      </div>
    </div>
  );
}

function ChatSpace({ tabId }: { tabId: string }) {
  const chatSpaceId = Number(tabId);
  const [detail, setDetail] = useState<ChatSpaceDetail | null>(null);
  const [notFound, setNotFound] = useState(false);
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const [model, setModel] = useState<ModelId>("qwen");
  const sendMessage = useChatStore((state) => state.sendMessage);
  const renameTab = useTabStore((state) => state.renameTab);
  const bottomRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Grows the input pill by one line per newline, up to MAX_VISIBLE_LINES, then
  // switches to an internal scrollbar — matches Claude's chat input behavior.
  const resizeTextarea = () => {
    const el = textareaRef.current;
    if (!el) return;
    const MAX_VISIBLE_LINES = 5;
    const lineHeight = parseFloat(getComputedStyle(el).lineHeight) || 20;
    const maxHeight = lineHeight * MAX_VISIBLE_LINES;

    el.style.height = "auto";
    const nextHeight = Math.min(el.scrollHeight, maxHeight);
    el.style.height = `${nextHeight}px`;
    el.style.overflowY = el.scrollHeight > maxHeight ? "auto" : "hidden";
  };

  useLayoutEffect(() => {
    resizeTextarea();
  }, [draft]);

  // The custom web font (Geist Variable) can finish loading after the layout
  // above already ran, which changes line-height and leaves the box sized for
  // the fallback font's smaller metrics — it looks "tiny" until something else
  // (like typing) triggers a resize. Recompute once the real font is ready.
  useEffect(() => {
    document.fonts?.ready.then(() => resizeTextarea());
  }, []);

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

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ block: "end" });
  }, [detail?.messages.length]);

  const handleSend = async () => {
    const content = draft.trim();
    if (!content || sending || notFound) return;
    setDraft("");
    setSending(true);
    try {
      const updated = await sendMessage(chatSpaceId, content);
      setDetail(updated);
      renameTab(tabId, updated.title);
    } finally {
      setSending(false);
    }
  };

  return (
    <div className="flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm">
      <div className="flex shrink-0 items-center border-b border-teal px-4 py-3">
        <h2 className="truncate text-sm font-semibold text-navy">
          {detail?.title ?? "New Chat"}
        </h2>
      </div>

      <div className="flex-1 overflow-y-auto px-4">
        {notFound ? (
          <p className="py-4 text-sm text-muted-foreground">
            This chat no longer exists. It may have been deleted.
          </p>
        ) : !detail || detail.messages.length === 0 ? (
          <p className="py-4 text-sm text-muted-foreground">
            Ask a question about your system.
          </p>
        ) : (
          detail.messages.map((message) => <MessageRow key={message.id} message={message} />)
        )}
        <div ref={bottomRef} />
      </div>

      <div className="shrink-0 p-3">
        <div className="flex flex-col gap-1 rounded-3xl border border-teal bg-canvas px-4 py-2">
          <textarea
            ref={textareaRef}
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                handleSend();
              }
            }}
            placeholder={
              notFound ? "This chat no longer exists" : "Ask about CPU, memory, processes…"
            }
            rows={1}
            disabled={notFound}
            className="min-h-5 resize-none bg-transparent py-1 text-sm text-navy outline-none placeholder:text-navy/40 disabled:cursor-not-allowed"
          />
          <div className="flex items-center justify-end gap-1">
            <DropdownMenu>
              <DropdownMenuTrigger className="flex cursor-pointer items-center gap-1 rounded-full px-2 py-1 text-xs text-navy/70 hover:bg-teal/25">
                {MODEL_LABELS[model]}
                <ChevronDown className="h-3 w-3" />
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end">
                <DropdownMenuRadioGroup
                  value={model}
                  onValueChange={(value) => setModel(value as ModelId)}
                >
                  <DropdownMenuRadioItem value="qwen">Qwen</DropdownMenuRadioItem>
                  <DropdownMenuRadioItem value="gpt-oss">GPT-OSS</DropdownMenuRadioItem>
                </DropdownMenuRadioGroup>
              </DropdownMenuContent>
            </DropdownMenu>

            <Button
              size="icon-sm"
              className="shrink-0 rounded-lg"
              onClick={handleSend}
              disabled={!draft.trim() || sending || notFound}
            >
              <ArrowUp className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}

export default ChatSpace;
