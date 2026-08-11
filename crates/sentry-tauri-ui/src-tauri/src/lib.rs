// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sentry_core::{
    ChatSpace, ChatSpaceDetail, ChatStore, HistoryStore, ProcessSamplePoint, SystemSamplePoint,
    TrackedArchive, TrackedProcess, TrackingStore,
};
use tauri::{AppHandle, Manager, State, Window};
use tauri_plugin_opener::OpenerExt;

struct MonitorState(Mutex<sentry_core::Monitor>);
struct HistoryState(Arc<HistoryStore>);
struct ChatState(ChatStore);
struct TrackingState(TrackingStore);

/// How often history is recorded to SQLite, independent of how often the frontend
/// polls `get_snapshot`. See [`sentry_core::spawn_recorder`].
const HISTORY_RECORD_INTERVAL: Duration = Duration::from_secs(2);

#[tauri::command]
fn get_snapshot(state: State<MonitorState>) -> sentry_core::SystemSnapshot {
    let mut monitor = state.0.lock().unwrap();
    monitor.snapshot()
}

#[tauri::command]
fn get_process_details(
    state: State<MonitorState>,
    pid: u32,
) -> Option<sentry_core::ProcessDetails> {
    let mut monitor = state.0.lock().unwrap();
    monitor.process_details(pid)
}

#[tauri::command]
fn get_system_timeline(
    state: State<HistoryState>,
    since_secs: u64,
) -> Result<Vec<SystemSamplePoint>, String> {
    state
        .0
        .system_timeline(Duration::from_secs(since_secs))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_process_timeline(
    state: State<HistoryState>,
    pid: u32,
    since_secs: u64,
) -> Result<Vec<ProcessSamplePoint>, String> {
    state
        .0
        .process_timeline(pid, Duration::from_secs(since_secs))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_app_timeline(
    state: State<HistoryState>,
    pids: Vec<i64>,
    since_secs: u64,
) -> Result<Vec<ProcessSamplePoint>, String> {
    let pids: Vec<u32> = pids.into_iter().map(|p| p as u32).collect();
    state
        .0
        .process_timeline_for_pids(&pids, Duration::from_secs(since_secs))
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn start_tracking(
    state: State<TrackingState>,
    name: String,
    pids: Vec<i64>,
) -> Result<TrackedProcess, String> {
    state.0.start(&name, &pids).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_tracked_processes(
    state: State<TrackingState>,
    name: Option<String>,
) -> Result<Vec<TrackedProcess>, String> {
    state.0.list(name.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_tracked_process(
    state: State<TrackingState>,
    id: i64,
) -> Result<Option<TrackedProcess>, String> {
    state.0.get(id).map_err(|e| e.to_string())
}

/// Ends a tracking session (idempotent — calling it again on an already-ended
/// session just returns the existing record) and, the first time, archives its
/// samples to a JSON file next to the history/chat SQLite databases so the
/// Details View can still show the feed after `process_samples` rows age out of
/// [`sentry_core::DEFAULT_RETENTION`].
#[tauri::command]
fn end_tracking(
    app: AppHandle,
    tracking: State<TrackingState>,
    history: State<HistoryState>,
    id: i64,
) -> Result<TrackedProcess, String> {
    let ended = tracking
        .0
        .end(id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "tracked process not found".to_string())?;

    if ended.archive_path.is_some() {
        return Ok(ended);
    }

    let pids: Vec<u32> = ended.pids.iter().map(|&p| p as u32).collect();
    let samples = history
        .0
        .process_timeline_for_pids(&pids, sentry_core::DEFAULT_RETENTION)
        .map_err(|e| e.to_string())?;

    let archive = TrackedArchive {
        id: ended.id,
        name: ended.name.clone(),
        pids: ended.pids.clone(),
        started_at_ms: ended.started_at_ms,
        ended_at_ms: ended.ended_at_ms.unwrap_or(ended.started_at_ms),
        samples,
    };

    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let archive_dir = data_dir.join("tracked_archives");
    std::fs::create_dir_all(&archive_dir).map_err(|e| e.to_string())?;
    let archive_path = archive_dir.join(format!("{id}.json"));
    let json = serde_json::to_string_pretty(&archive).map_err(|e| e.to_string())?;
    std::fs::write(&archive_path, json).map_err(|e| e.to_string())?;

    tracking
        .0
        .set_archive_path(id, &archive_path.to_string_lossy())
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "tracked process not found".to_string())
}

#[tauri::command]
fn delete_tracked_process(tracking: State<TrackingState>, id: i64) -> Result<bool, String> {
    let deleted = tracking.0.delete(id).map_err(|e| e.to_string())?;
    if let Some(tracked) = &deleted {
        if let Some(path) = &tracked.archive_path {
            let _ = std::fs::remove_file(path);
        }
    }
    Ok(deleted.is_some())
}

#[tauri::command]
fn get_tracked_archive(
    tracking: State<TrackingState>,
    id: i64,
) -> Result<Option<TrackedArchive>, String> {
    let Some(tracked) = tracking.0.get(id).map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    let Some(path) = tracked.archive_path else {
        return Ok(None);
    };
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let archive: TrackedArchive = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(Some(archive))
}

#[tauri::command]
fn create_chat_space(state: State<ChatState>) -> Result<ChatSpace, String> {
    state.0.create_chat_space().map_err(|e| e.to_string())
}

#[tauri::command]
fn list_chat_spaces(state: State<ChatState>) -> Result<Vec<ChatSpace>, String> {
    state.0.list_chat_spaces().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_chat_space(state: State<ChatState>, id: i64) -> Result<Option<ChatSpaceDetail>, String> {
    state.0.get_chat_space(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_chat_space(state: State<ChatState>, id: i64) -> Result<bool, String> {
    state.0.delete_chat_space(id).map_err(|e| e.to_string())
}

#[tauri::command]
fn send_chat_message(
    state: State<ChatState>,
    chat_space_id: i64,
    content: String,
) -> Result<ChatSpaceDetail, String> {
    state
        .0
        .send_message(chat_space_id, &content)
        .map_err(|e| e.to_string())
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
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let history = Arc::new(HistoryStore::open(data_dir.join("history.sqlite"))?);
            // Runs for the process lifetime, independent of whether the frontend is
            // polling `get_snapshot` — see sentry_core::spawn_recorder.
            let _recorder = sentry_core::spawn_recorder(history.clone(), HISTORY_RECORD_INTERVAL);
            app.manage(HistoryState(history));
            app.manage(ChatState(ChatStore::open(data_dir.join("chat.sqlite"))?));
            app.manage(TrackingState(TrackingStore::open(
                data_dir.join("tracking.sqlite"),
            )?));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            close_window,
            minimize_window,
            maximize_window,
            get_snapshot,
            get_process_details,
            get_system_timeline,
            get_process_timeline,
            get_app_timeline,
            start_tracking,
            list_tracked_processes,
            get_tracked_process,
            end_tracking,
            delete_tracked_process,
            get_tracked_archive,
            create_chat_space,
            list_chat_spaces,
            get_chat_space,
            delete_chat_space,
            send_chat_message,
            write_and_open_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
