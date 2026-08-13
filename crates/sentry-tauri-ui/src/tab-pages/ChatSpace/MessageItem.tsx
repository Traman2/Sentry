import { Check, Copy, Timer } from "lucide-react";
import { useState, type ReactNode } from "react";
import { Bubble, BubbleContent } from "@/components/ui/bubble";
import { Button } from "@/components/ui/button";
import {
  Message,
  MessageContent,
  MessageFooter,
} from "@/components/ui/message";
import { Skeleton } from "@/components/ui/skeleton";
import { Spinner } from "@/components/ui/spinner";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import type { ChatMessage } from "@/store/chat";
import { formatMessageTime, formatThinkingDuration, mockThinkingDurationMs } from "./time";

function CopyAction({ content }: { content: string }) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    await navigator.clipboard.writeText(content);
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  };

  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <Button
            variant="ghost"
            size="icon-xs"
            aria-label="Copy reply"
            onClick={handleCopy}
            className="text-navy/50 hover:bg-teal/25 hover:text-navy"
          />
        }
      >
        {copied ? <Check className="size-3" /> : <Copy className="size-3" />}
      </TooltipTrigger>
      <TooltipContent>{copied ? "Copied" : "Copy reply"}</TooltipContent>
    </Tooltip>
  );
}

/** How long the agent spent on this reply. Sits in the hover footer beside the
 * copy button and borrows its metrics, so the two read as one row of muted
 * afterthoughts rather than a control next to a label. */
function ThinkingTime({ durationMs }: { durationMs: number }) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <span className="inline-flex h-6 items-center gap-1 rounded-[min(var(--radius-md),10px)] px-2 text-xs text-navy/50 tabular-nums" />
        }
      >
        <Timer className="size-3" />
        {formatThinkingDuration(durationMs)}
      </TooltipTrigger>
      <TooltipContent>Agent took {formatThinkingDuration(durationMs)} to respond</TooltipContent>
    </Tooltip>
  );
}

/** A user turn: left-aligned like everything else, but wrapped in a tinted
 * bubble so a question is instantly distinguishable from the long-form answer
 * that follows it. */
export function UserMessage({ message }: { message: ChatMessage }) {
  return (
    <Message align="start">
      <MessageContent>
        <Bubble variant="outline">
          <BubbleContent className="rounded-2xl border-teal/25 px-3.5 py-2.5 whitespace-pre-wrap text-navy">
            {message.content}
          </BubbleContent>
        </Bubble>
      </MessageContent>
    </Message>
  );
}

/** An assistant turn: full width and unbubbled, because replies run long and
 * carry rich blocks (charts, tables) that a bubble would crop. `children` is
 * that rich-block slot — it renders under the prose, inside the same column. */
export function AssistantMessage({
  message,
  children,
}: {
  message: ChatMessage;
  children?: ReactNode;
}) {
  return (
    <Message align="start" className="flex-col gap-2">
      <MessageContent className="gap-3 text-sm leading-relaxed text-navy">
        <div className="whitespace-pre-wrap wrap-break-word">{message.content}</div>
        {children}
      </MessageContent>
      <MessageFooter className="gap-1 px-0 opacity-0 transition-opacity group-hover/message:opacity-100">
        <CopyAction content={message.content} />
        <ThinkingTime durationMs={mockThinkingDurationMs(message.id)} />
        <span className="text-[11px] text-navy/40 tabular-nums">
          {formatMessageTime(message.created_at_ms)}
        </span>
      </MessageFooter>
    </Message>
  );
}

/** Shape-matched placeholder for the whole list while the chat's history is
 * still being fetched — a short user bubble followed by a block of reply. */
export function MessageListSkeleton() {
  return (
    <div className="flex flex-col gap-7">
      <Skeleton className="h-9 w-56 rounded-2xl" />
      <div className="flex flex-col gap-2.5">
        <Skeleton className="h-5 w-20 rounded-md" />
        <Skeleton className="h-3 w-full max-w-lg" />
        <Skeleton className="h-3 w-full max-w-md" />
        <Skeleton className="h-3 w-40" />
      </div>
    </div>
  );
}

/** Placeholder assistant turn shown while the backend is generating, so the
 * conversation never sits visually frozen after the user hits send. */
export function PendingAssistantMessage() {
  return (
    <Message align="start" className="flex-col gap-2">
      <MessageContent className="gap-2.5">
        <span className="flex items-center gap-2 text-sm text-muted-foreground">
          <Spinner className="size-3.5" />
          Reading your system…
        </span>
        <Skeleton className="h-3 w-full max-w-lg" />
        <Skeleton className="h-3 w-full max-w-md" />
        <Skeleton className="h-3 w-32" />
      </MessageContent>
    </Message>
  );
}
