const MINUTE_MS = 60_000;
const HOUR_MS = 60 * MINUTE_MS;
const DAY_MS = 24 * HOUR_MS;

/** Compact "how long ago" — "just now", "5m ago", "3h ago", "2d ago", then an
 * absolute date once it's beyond a week. */
export function formatRelativeTime(timestampMs: number): string {
  const elapsed = Date.now() - timestampMs;

  if (elapsed < MINUTE_MS) return "just now";
  if (elapsed < HOUR_MS) return `${Math.floor(elapsed / MINUTE_MS)}m ago`;
  if (elapsed < DAY_MS) return `${Math.floor(elapsed / HOUR_MS)}h ago`;
  if (elapsed < 7 * DAY_MS) return `${Math.floor(elapsed / DAY_MS)}d ago`;

  return new Date(timestampMs).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
  });
}

/** Which sidebar heading a timestamp belongs under. Buckets are relative to
 * calendar day boundaries, not elapsed hours, so something from 11pm last
 * night reads as "Yesterday" rather than "3h ago". */
export function recencyBucket(timestampMs: number): string {
  const now = new Date();
  const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();

  if (timestampMs >= startOfToday) return "Today";
  if (timestampMs >= startOfToday - DAY_MS) return "Yesterday";
  if (timestampMs >= startOfToday - 7 * DAY_MS) return "Previous 7 days";
  if (timestampMs >= startOfToday - 30 * DAY_MS) return "Previous 30 days";

  return new Date(timestampMs).toLocaleDateString(undefined, {
    month: "long",
    year: "numeric",
  });
}

/** Splits an already newest-first list into consecutive {@link recencyBucket}
 * runs, for rendering a chat-style sidebar grouped by date. Input order is
 * preserved; an unsorted list would produce repeated headings. */
export function groupByRecency<T>(
  items: T[],
  getTimestampMs: (item: T) => number,
): { label: string; items: T[] }[] {
  const groups: { label: string; items: T[] }[] = [];

  for (const item of items) {
    const label = recencyBucket(getTimestampMs(item));
    const current = groups[groups.length - 1];

    if (current && current.label === label) current.items.push(item);
    else groups.push({ label, items: [item] });
  }

  return groups;
}
