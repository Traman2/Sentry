use std::process::{Child, Command, Stdio};

use super::Monitor;

/// Spawns a throwaway child that stays alive long enough to be killed, without needing a
/// console or stdin (so it works under a headless test runner).
fn spawn_victim() -> Child {
    #[cfg(windows)]
    let mut command = {
        let mut c = Command::new("cmd");
        c.args(["/C", "ping", "-n", "30", "127.0.0.1"]);
        c
    };
    #[cfg(not(windows))]
    let mut command = {
        let mut c = Command::new("sleep");
        c.arg("30");
        c
    };

    command
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn the victim process")
}

#[test]
fn kill_process_terminates_a_running_process() {
    let mut victim = spawn_victim();
    let pid = victim.id();
    let mut monitor = Monitor::new();

    let outcome = monitor.kill_process(pid, None, None);

    assert!(outcome.found, "victim not found: {}", outcome.message);
    assert!(outcome.delivered, "not delivered: {}", outcome.message);
    assert_eq!(outcome.pid, pid);
    assert_eq!(outcome.signal, "Kill");

    // Reap before asking the OS whether it's gone — an unwaited child lingers as a zombie
    // and would still show up in the process table.
    victim.wait().expect("failed to wait on the victim");

    let after = monitor.kill_process(pid, None, None);
    assert!(!after.found, "victim survived: {}", after.message);
}

#[test]
fn kill_process_refuses_when_expect_name_does_not_match() {
    let mut victim = spawn_victim();
    let pid = victim.id();
    let mut monitor = Monitor::new();

    let outcome = monitor.kill_process(pid, Some("definitely-not-the-real-name"), None);

    assert!(outcome.found);
    assert!(
        !outcome.delivered,
        "a name mismatch must not kill anything: {}",
        outcome.message
    );
    assert!(
        outcome.message.contains("recycled"),
        "message should explain the pid-reuse guard, got: {}",
        outcome.message
    );

    // The guard must have left it running.
    assert!(
        victim.try_wait().expect("try_wait failed").is_none(),
        "victim was killed despite the name mismatch"
    );

    let _ = victim.kill();
    let _ = victim.wait();
}

#[test]
fn kill_process_reports_a_missing_pid_rather_than_failing() {
    let mut monitor = Monitor::new();

    let outcome = monitor.kill_process(u32::MAX, None, None);

    assert!(!outcome.found);
    assert!(!outcome.delivered);
    assert!(outcome.name.is_none());
    assert!(outcome.message.contains("no process"));
}
