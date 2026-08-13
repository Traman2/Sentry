import { MessageSquareOff } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { SUGGESTED_PROMPTS } from "./constants";

/** First-run state for a chat with no messages: a short pitch plus four
 * one-click starters, so the page never opens as a blank column. */
export function ChatEmptyState({ onPick }: { onPick: (prompt: string) => void }) {
  return (
    <Empty className="border-none">
      <EmptyHeader>
        <EmptyMedia
          variant="icon"
          className="size-10 rounded-xl bg-white text-navy"
        >
          <img className="size-8" src="/sentry-logo.svg" alt="app logo"/>
        </EmptyMedia>
        <EmptyTitle className="text-lg">Ask about your system</EmptyTitle>
        <EmptyDescription>
          Sentry reads live CPU, memory, disk and process data to answer questions
          and chart what it finds.
        </EmptyDescription>
      </EmptyHeader>

      <EmptyContent className="max-w-xl">
        <div className="grid w-full gap-2 sm:grid-cols-2">
          {SUGGESTED_PROMPTS.map(({ icon: Icon, label, prompt }) => (
            <Button
              key={label}
              variant="outline"
              onClick={() => onPick(prompt)}
              className="h-auto justify-start gap-2.5 border-teal/40 px-3 py-2.5 text-left text-xs font-medium whitespace-normal text-navy hover:border-teal hover:bg-teal/10"
            >
              <Icon className="size-3.5 shrink-0 text-teal" />
              {label}
            </Button>
          ))}
        </div>
      </EmptyContent>
    </Empty>
  );
}

/** Shown when the chat space behind this tab has been deleted from the sidebar
 * while the tab stayed open. */
export function ChatMissingState() {
  return (
    <Empty className="border-none">
      <EmptyHeader>
        <EmptyMedia
          variant="icon"
          className="size-10 rounded-xl bg-destructive/10 text-destructive"
        >
          <MessageSquareOff className="size-5" />
        </EmptyMedia>
        <EmptyTitle className="text-lg">Chat unavailable</EmptyTitle>
        <EmptyDescription>
          This conversation no longer exists. It may have been deleted from the
          chat list.
        </EmptyDescription>
      </EmptyHeader>
    </Empty>
  );
}
