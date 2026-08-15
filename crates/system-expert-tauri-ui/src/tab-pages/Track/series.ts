import type { ProcessSamplePoint } from "../ResourceMonitor/types";

/**
 * Sums `metric` across every tracked pid at each sampled instant, into one
 * combined line — an app being tracked is a fixed set of pids, and plotting
 * one line per pid gets unreadable past a handful of processes, so we always
 * chart the app/process's total instead.
 */
export function buildTotalSeries(
  samples: ProcessSamplePoint[],
  metric: "cpu_usage_percent" | "memory_bytes",
) {
  const totalsByTime = new Map<number, number>();
  for (const sample of samples) {
    totalsByTime.set(
      sample.timestamp_ms,
      (totalsByTime.get(sample.timestamp_ms) ?? 0) + sample[metric],
    );
  }
  return Array.from(totalsByTime.entries())
    .map(([timestamp_ms, total]) => ({ timestamp_ms, total }))
    .sort((a, b) => a.timestamp_ms - b.timestamp_ms);
}

/** Same idea as {@link buildTotalSeries}, but for disk I/O — combines read and
 * write into two lines on one chart rather than a separate chart each. */
export function buildStorageSeries(samples: ProcessSamplePoint[]) {
  const byTime = new Map<number, { read: number; write: number }>();
  for (const sample of samples) {
    const bucket = byTime.get(sample.timestamp_ms) ?? { read: 0, write: 0 };
    bucket.read += sample.disk_read_bytes_per_sec;
    bucket.write += sample.disk_written_bytes_per_sec;
    byTime.set(sample.timestamp_ms, bucket);
  }
  return Array.from(byTime.entries())
    .map(([timestamp_ms, { read, write }]) => ({ timestamp_ms, read, write }))
    .sort((a, b) => a.timestamp_ms - b.timestamp_ms);
}

const TABLE_BUCKET_MS = 5000;

/** Downsamples the raw ~2s-interval samples to one row per pid per 5-second
 * bucket (the latest sample in that bucket), newest first — the underlying
 * `process_samples` recording cadence is finer than a human wants to read in a
 * table, and this is what the range toggle (1 min/10 min/24 hr) actually
 * generates rows over. Multiple tracked pids just interleave as separate rows
 * at the same bucketed timestamp. */
export function bucketSamplesForTable(samples: ProcessSamplePoint[]): ProcessSamplePoint[] {
  const latestByBucket = new Map<string, ProcessSamplePoint>();
  for (const sample of samples) {
    const bucketStart = Math.floor(sample.timestamp_ms / TABLE_BUCKET_MS) * TABLE_BUCKET_MS;
    const key = `${bucketStart}:${sample.pid}`;
    const current = latestByBucket.get(key);
    if (!current || sample.timestamp_ms > current.timestamp_ms) {
      latestByBucket.set(key, { ...sample, timestamp_ms: bucketStart });
    }
  }
  return Array.from(latestByBucket.values()).sort((a, b) => b.timestamp_ms - a.timestamp_ms);
}

export function filterToRange(samples: ProcessSamplePoint[], sinceSecs: number, anchorMs: number) {
  const cutoff = anchorMs - sinceSecs * 1000;
  return samples.filter((s) => s.timestamp_ms >= cutoff);
}
