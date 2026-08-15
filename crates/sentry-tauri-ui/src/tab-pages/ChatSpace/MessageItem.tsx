import { Check, ChevronRight, Copy, OctagonX, Timer, TriangleAlert } from "lucide-react";
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
import { Markdown } from "./Markdown";
import { formatMessageTime, formatThinkingDuration } from "./time";

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
  respondedToMessage,
  children,
}: {
  message: ChatMessage;
  /** The user turn this reply answers — its `created_at_ms` is the start of the timer shown
   * in the footer. Omitted (rather than guessed) when there's no matching turn to time
   * against, e.g. a transcript that doesn't start on a user message. */
  respondedToMessage?: ChatMessage;
  children?: ReactNode;
}) {
  const thinkingDurationMs = respondedToMessage
    ? message.created_at_ms - respondedToMessage.created_at_ms
    : undefined;

  return (
    <Message align="start" className="flex-col gap-2">
      <MessageContent className="gap-3 text-sm leading-relaxed text-navy">
        <Markdown content={message.content} />
        {children}
      </MessageContent>
      <MessageFooter className="gap-1 px-0 opacity-0 transition-opacity group-hover/message:opacity-100">
        <CopyAction content={message.content} />
        {thinkingDurationMs !== undefined && thinkingDurationMs >= 0 && (
          <ThinkingTime durationMs={thinkingDurationMs} />
        )}
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

/** A failed agent turn. Shows the one-line cause, with the full trace behind a disclosure —
 * the summary is what the user acts on, the trace is what they'd paste into an issue. */
export function ErrorMessage({ message }: { message: ChatMessage }) {
  return (
    <Message align="start" className="flex-col gap-2">
      <MessageContent className="gap-0">
        <div className="rounded-lg border border-destructive/30 bg-destructive/5">
          <div className="flex items-start gap-2 px-3 py-2.5">
            <TriangleAlert className="mt-px size-3.5 shrink-0 text-destructive" />
            <div className="min-w-0 flex-1">
              <p className="text-sm text-destructive">Couldn’t answer that.</p>
              <p className="mt-0.5 text-xs wrap-break-word text-navy/60">{message.content}</p>
            </div>
          </div>

          {message.details && (
            // A native <details> disclosure: keyboard- and screen-reader-accessible
            // without a component, and it keeps its own open/closed state across the
            // re-renders a new message causes.
            <details className="group/details border-t border-destructive/20">
              <summary className="flex cursor-pointer list-none items-center gap-1 px-3 py-1.5 text-[11px] text-navy/50 transition-colors hover:text-navy [&::-webkit-details-marker]:hidden">
                <ChevronRight className="size-3 transition-transform group-open/details:rotate-90" />
                View details
              </summary>
              <pre className="max-h-64 overflow-auto px-3 pb-2.5 text-[11px] leading-relaxed whitespace-pre-wrap text-navy/60">
                {message.details}
              </pre>
            </details>
          )}
        </div>
      </MessageContent>
      <MessageFooter className="gap-1 px-0 opacity-0 transition-opacity group-hover/message:opacity-100">
        <CopyAction content={message.details ?? message.content} />
        <span className="text-[11px] text-navy/40 tabular-nums">
          {formatMessageTime(message.created_at_ms)}
        </span>
      </MessageFooter>
    </Message>
  );
}

/** A turn the user stopped. Deliberately quiet — this is an outcome they chose, not a
 * failure, so it reads as a note in the transcript rather than an alarm. */
export function InterruptedMessage({ message }: { message: ChatMessage }) {
  return (
    <Message align="start" className="flex-col gap-2">
      <MessageContent>
        <span className="inline-flex items-center gap-2 text-xs text-navy/50">
          <OctagonX className="size-3.5 shrink-0" />
          {message.content}
        </span>
      </MessageContent>
    </Message>
  );
}

/** Placeholder assistant turn shown while the agent is working, so the
 * conversation never sits visually frozen after the user hits send.
 *
 * The reply comes from a separate agent process, which can be down — a missing API key, a
 * crash. Spinning forever would leave the user waiting on something that is never coming,
 * so `agentDown` swaps the spinner for the reason. */
export function PendingAssistantMessage({
  agentDown = false,
  detail,
  onStop,
  steps = [],
}: {
  agentDown?: boolean;
  detail?: string;
  /** Abandons the turn. Present whenever a reply is outstanding, because the case this
   * exists for is precisely the one where the agent has stopped responding on its own. */
  onStop?: () => void;
  /** What the agent has done so far, oldest first. Transient — this whole component is
   * replaced by the reply once it lands. */
  steps?: string[];
}) {
  if (agentDown) {
    return (
      <Message align="start" className="flex-col gap-2">
        <MessageContent className="gap-1.5">
          <span className="flex items-center gap-2 text-sm text-destructive">
            <TriangleAlert className="size-3.5" />
            The agent isn’t running, so this won’t be answered.
          </span>
          {detail && <span className="text-xs text-navy/50">{detail}</span>}
        </MessageContent>
      </Message>
    );
  }

  return (
    <Message align="start" className="flex-col gap-2">
      <MessageContent className="gap-2.5">
        <span className="flex items-center gap-2 text-sm text-muted-foreground">
          <Spinner className="size-3.5" />
          {/* The newest step is the live one; before any arrive, say something generic
              rather than leave the row empty. */}
          {steps.length > 0 ? steps[steps.length - 1] : "Reading your system…"}
          {onStop && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onStop}
              className="h-6 gap-1 px-2 text-xs text-navy/50 hover:bg-destructive/10 hover:text-destructive"
            >
              <OctagonX className="size-3" />
              Stop
            </Button>
          )}
        </span>

        {steps.length > 1 ? (
          // Everything already done, oldest first, so the trail of work reads top-down and
          // the live step above stays the last line.
          <ol className="flex flex-col gap-1 border-l border-teal/40 pl-3">
            {steps.slice(0, -1).map((step, index) => (
              <li
                key={`${index}-${step}`}
                className="text-xs wrap-break-word text-navy/45"
              >
                {step}
              </li>
            ))}
          </ol>
        ) : (
          <>
            <Skeleton className="h-3 w-full max-w-lg" />
            <Skeleton className="h-3 w-full max-w-md" />
            <Skeleton className="h-3 w-32" />
          </>
        )}
      </MessageContent>
    </Message>
  );
}
