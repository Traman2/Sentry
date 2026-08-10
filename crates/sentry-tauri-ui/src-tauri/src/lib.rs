// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use tauri::{State, Window};

struct MonitorState(Mutex<sentry_core::Monitor>);

#[tauri::command]
fn get_snapshot(state: State<MonitorState>) -> sentry_core::SystemSnapshot {
    let mut monitor = state.0.lock().unwrap();
    monitor.snapshot()
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn close_window(window: Window) {
    let _ = window.close();
}

#[tauri::command]
fn minimize_window(window: Window) {
    let _ = window.minimize();
}

#[tauri::command]
fn maximize_window(window: Window) {
    if let Ok(is_maximized) = window.is_maximized() {
        if is_maximized {
            let _ = window.unmaximize();
        } else {
            let _ = window.maximize();
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(MonitorState(Mutex::new(sentry_core::Monitor::new())))
        .invoke_handler(tauri::generate_handler![
            greet,
            close_window,
            minimize_window,
            maximize_window,
            get_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
