//! Continuous snapshot delivery, decoupled from any particular transport.

use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use super::Monitor;

/// Runs the monitor loop on the calling thread, invoking `on_snapshot` with each
/// snapshot serialized as a JSON string. Stops when `on_snapshot` returns `false`.
pub fn stream_json<F>(interval: Duration, mut on_snapshot: F)
where
    F: FnMut(String) -> bool,
{
    let mut monitor = Monitor::new();
    loop {
        match monitor.snapshot_json() {
            Ok(json) => {
                if !on_snapshot(json) {
                    break;
                }
            }
            Err(_) => continue,
        }
        thread::sleep(interval);
    }
}

/// Spawns the monitor loop on a background thread and returns a channel that yields
/// one JSON snapshot per interval. Consumers (a Tauri command emitting events, an MCP
/// resource, etc.) drive their own transport by draining this receiver, so
/// `system-expert-core` stays independent of any particular async runtime or streaming
/// protocol.
pub fn spawn_stream(interval: Duration) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        stream_json(interval, |json| tx.send(json).is_ok());
    });
    rx
}
