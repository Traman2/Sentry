//! Bucket-averaging history down to a token-friendly point count.

use sentry_core::{ProcessSamplePoint, SystemSamplePoint};

/// Bucket-averages `points` down to at most `max_points`, preserving chronological order.
///
/// The recorder samples every 2s and retains 24h, so an ungated "show me the last day"
/// query returns ~43,200 points. Averaging into buckets keeps the shape of the curve —
/// which is all an agent needs to answer "was this spiking an hour ago" — at a size that
/// fits in a tool result.
pub fn downsample_system(
    points: Vec<SystemSamplePoint>,
    max_points: usize,
) -> Vec<SystemSamplePoint> {
    bucket(points, max_points, |chunk| {
        let n = chunk.len() as f64;
        SystemSamplePoint {
            // The bucket's last timestamp, so the final point is the most recent reading
            // rather than an average that lands slightly in the past.
            timestamp_ms: chunk[chunk.len() - 1].timestamp_ms,
            cpu_usage_percent: (chunk
                .iter()
                .map(|p| p.cpu_usage_percent as f64)
                .sum::<f64>()
                / n) as f32,
            used_memory_bytes: (chunk
                .iter()
                .map(|p| p.used_memory_bytes as f64)
                .sum::<f64>()
                / n) as u64,
            total_memory_bytes: chunk[chunk.len() - 1].total_memory_bytes,
            used_swap_bytes: (chunk.iter().map(|p| p.used_swap_bytes as f64).sum::<f64>() / n)
                as u64,
            network_rx_bytes_per_sec: (chunk
                .iter()
                .map(|p| p.network_rx_bytes_per_sec as f64)
                .sum::<f64>()
                / n) as u64,
            network_tx_bytes_per_sec: (chunk
                .iter()
                .map(|p| p.network_tx_bytes_per_sec as f64)
                .sum::<f64>()
                / n) as u64,
        }
    })
}

/// Bucket-averages process samples down to at most `max_points` **per pid**.
///
/// Grouping by pid first is not optional: a tracked session covers several pids at once, and
/// averaging across a chunk that straddles two different processes would invent readings
/// belonging to neither.
pub fn downsample_process(
    points: Vec<ProcessSamplePoint>,
    max_points: usize,
) -> Vec<ProcessSamplePoint> {
    let mut by_pid: Vec<(u32, Vec<ProcessSamplePoint>)> = Vec::new();
    for point in points {
        match by_pid.iter_mut().find(|(pid, _)| *pid == point.pid) {
            Some((_, group)) => group.push(point),
            None => by_pid.push((point.pid, vec![point])),
        }
    }

    let mut out: Vec<ProcessSamplePoint> = by_pid
        .into_iter()
        .flat_map(|(_, group)| {
            bucket(group, max_points, |chunk| {
                let n = chunk.len() as f64;
                let last = &chunk[chunk.len() - 1];
                ProcessSamplePoint {
                    timestamp_ms: last.timestamp_ms,
                    pid: last.pid,
                    name: last.name.clone(),
                    cpu_usage_percent: (chunk
                        .iter()
                        .map(|p| p.cpu_usage_percent as f64)
                        .sum::<f64>()
                        / n) as f32,
                    memory_bytes: (chunk.iter().map(|p| p.memory_bytes as f64).sum::<f64>() / n)
                        as u64,
                    disk_read_bytes_per_sec: (chunk
                        .iter()
                        .map(|p| p.disk_read_bytes_per_sec as f64)
                        .sum::<f64>()
                        / n) as u64,
                    disk_written_bytes_per_sec: (chunk
                        .iter()
                        .map(|p| p.disk_written_bytes_per_sec as f64)
                        .sum::<f64>()
                        / n) as u64,
                }
            })
        })
        .collect();

    out.sort_by_key(|p| (p.timestamp_ms, p.pid));
    out
}

/// Splits `points` into at most `max_points` contiguous chunks and folds each with `reduce`.
fn bucket<T, F>(points: Vec<T>, max_points: usize, reduce: F) -> Vec<T>
where
    F: Fn(&[T]) -> T,
{
    if max_points == 0 || points.len() <= max_points {
        return points;
    }
    // Ceiling division: with 43,200 points and max 120 this is 360 per bucket, yielding
    // exactly 120 buckets. Rounding down instead would overshoot `max_points`.
    let chunk_size = points.len().div_ceil(max_points);
    points.chunks(chunk_size).map(reduce).collect()
}

#[cfg(test)]
#[path = "downsample_tests.rs"]
mod tests;
