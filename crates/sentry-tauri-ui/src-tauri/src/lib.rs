// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::Mutex;
use tauri::{AppHandle, State, Window};
use tauri_plugin_opener::OpenerExt;

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
fn write_and_open_file(app: AppHandle, path: String, contents: String) -> Result<(), String> {
    std::fs::write(&path, contents).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(&path, None::<&str>)
        .map_err(|e| e.to_string())
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
        .plugin(tauri_plugin_dialog::init())
        .manage(MonitorState(Mutex::new(sentry_core::Monitor::new())))
        .invoke_handler(tauri::generate_handler![
            greet,
            close_window,
            minimize_window,
            maximize_window,
            get_snapshot,
            write_and_open_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
