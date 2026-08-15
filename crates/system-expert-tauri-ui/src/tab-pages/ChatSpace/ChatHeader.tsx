/** The chat page's top bar: just the conversation's identity plus its size.
 * Every control that acts on the conversation lives in the composer, so this
 * bar stays a label rather than a toolbar. */
export function ChatHeader({
  title,
  messageCount,
}: {
  title: string;
  messageCount: number;
}) {
  return (
    <div className="flex shrink-0 items-center gap-3 border-b border-teal/50 px-4 py-2.5">
      <h2 className="min-w-0 flex-1 truncate text-sm font-medium text-navy">{title}</h2>
      {messageCount > 0 && (
        <span className="shrink-0 text-[11px] text-muted-foreground tabular-nums">
          {messageCount === 1 ? "1 message" : `${messageCount} messages`}
        </span>
      )}
    </div>
  );
}
