/** Clock time for a message, e.g. "14:32" — shown in the hover footer under an
 * assistant reply. Chats are read top-to-bottom in one sitting, so the time of
 * day is more useful here than a relative "3 minutes ago". */
export function formatMessageTime(timestampMs: number): string {
  return new Date(timestampMs).toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}
