use super::*;

#[test]
fn page_processes_reports_totals_independent_of_the_page() {
    let rows: Vec<ProcessRow> = (0..50).map(|i| row(i, "chrome.exe", i as f32)).collect();
    let page = page_processes(&rows, ProcessSort::Cpu, None, None, 10, 0);

    assert_eq!(page.total_processes, 50);
    assert_eq!(page.total_matched, 50);
    assert_eq!(page.returned, 10);
    // Sorted by CPU descending, so the highest pid (highest cpu) leads.
    assert_eq!(page.processes[0].pid, 49);
}

#[test]
fn page_processes_filters_by_name_before_counting_matches() {
    let rows = vec![
        row(1, "chrome.exe", 5.0),
        row(2, "notepad.exe", 1.0),
        row(3, "Chrome.exe", 9.0),
    ];
    let page = page_processes(&rows, ProcessSort::Cpu, Some("chrome"), None, 10, 0);

    assert_eq!(page.total_processes, 3);
    assert_eq!(page.total_matched, 2, "match should be case-insensitive");
    assert_eq!(page.processes[0].pid, 3);
}

fn row(pid: u32, name: &str, cpu: f32) -> ProcessRow {
    ProcessRow {
        pid,
        parent_pid: None,
        name: name.to_string(),
        executable_path: None,
        command: String::new(),
        status: "Run".to_string(),
        user: None,
        effective_user: None,
        group_id: None,
        effective_group_id: None,
        session_id: None,
        thread_count: None,
        start_time_unix_secs: 0,
        run_time_secs: 0,
        cpu_usage_percent: cpu,
        cpu_usage_display: String::new(),
        memory_bytes: 0,
        memory_display: String::new(),
        memory_percent: 0.0,
        memory_percent_display: String::new(),
        virtual_memory_bytes: 0,
        virtual_memory_display: String::new(),
        disk_read_bytes_per_sec: 0,
        disk_read_display: String::new(),
        disk_written_bytes_per_sec: 0,
        disk_write_display: String::new(),
        disk_total_read_bytes: 0,
        disk_total_read_display: String::new(),
        disk_total_written_bytes: 0,
        disk_total_write_display: String::new(),
    }
}
