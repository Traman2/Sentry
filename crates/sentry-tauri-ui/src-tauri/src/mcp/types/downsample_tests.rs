use super::*;

fn sys_point(timestamp_ms: i64, cpu: f32) -> SystemSamplePoint {
    SystemSamplePoint {
        timestamp_ms,
        cpu_usage_percent: cpu,
        used_memory_bytes: 100,
        total_memory_bytes: 1000,
        used_swap_bytes: 0,
        network_rx_bytes_per_sec: 10,
        network_tx_bytes_per_sec: 20,
    }
}

fn proc_point(timestamp_ms: i64, pid: u32, cpu: f32) -> ProcessSamplePoint {
    ProcessSamplePoint {
        timestamp_ms,
        pid,
        name: format!("p{pid}"),
        cpu_usage_percent: cpu,
        memory_bytes: 100,
        disk_read_bytes_per_sec: 0,
        disk_written_bytes_per_sec: 0,
    }
}

#[test]
fn downsample_leaves_short_series_untouched() {
    let points = vec![sys_point(1, 10.0), sys_point(2, 20.0)];
    let out = downsample_system(points, 120);
    assert_eq!(out.len(), 2);
    assert_eq!(out[1].cpu_usage_percent, 20.0);
}

#[test]
fn downsample_never_exceeds_max_points() {
    let points: Vec<_> = (0..43_200).map(|i| sys_point(i, 50.0)).collect();
    let out = downsample_system(points, 120);
    assert!(out.len() <= 120, "got {} points", out.len());
}

#[test]
fn downsample_averages_within_a_bucket_and_keeps_the_last_timestamp() {
    let points = vec![
        sys_point(1, 0.0),
        sys_point(2, 100.0),
        sys_point(3, 0.0),
        sys_point(4, 100.0),
    ];
    let out = downsample_system(points, 2);
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].cpu_usage_percent, 50.0);
    assert_eq!(out[0].timestamp_ms, 2);
    assert_eq!(out[1].timestamp_ms, 4);
}

#[test]
fn downsample_process_never_averages_across_two_pids() {
    // Interleaved pids: a naive chunker would blend 10.0 and 90.0 into 50.0 for both.
    let points = vec![
        proc_point(1, 100, 10.0),
        proc_point(1, 200, 90.0),
        proc_point(2, 100, 10.0),
        proc_point(2, 200, 90.0),
    ];
    let out = downsample_process(points, 1);

    assert_eq!(out.len(), 2);
    let p100 = out.iter().find(|p| p.pid == 100).unwrap();
    let p200 = out.iter().find(|p| p.pid == 200).unwrap();
    assert_eq!(p100.cpu_usage_percent, 10.0);
    assert_eq!(p200.cpu_usage_percent, 90.0);
}
