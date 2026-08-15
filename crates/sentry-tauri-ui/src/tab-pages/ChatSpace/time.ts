/** Clock time for a message, e.g. "14:32" — shown in the hover footer under an
 * assistant reply. Chats are read top-to-bottom in one sitting, so the time of
 * day is more useful here than a relative "3 minutes ago". */
export function formatMessageTime(timestampMs: number): string {
  return new Date(timestampMs).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** How long the agent worked on a reply, e.g. "4.2s", "18s", "1m 07s". Sub-ten
 * seconds keeps a decimal because that's the range where the difference between
 * a cached answer and a real one is visible. */
export function formatThinkingDuration(durationMs: number): string {
  const seconds = durationMs / 1000;
  if (seconds < 10) return `${seconds.toFixed(1)}s`;
  if (seconds < 60) return `${Math.round(seconds)}s`;
  const minutes = Math.floor(seconds / 60);
  return `${minutes}m ${String(Math.round(seconds % 60)).padStart(2, "0")}s`;
}
