// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod agent;
mod archive;
mod mcp;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use sentry_core::{
    ChatSpace, ChatSpaceDetail, ChatStore, HistoryStore, KillOutcome, ProcessSamplePoint,
    SystemSamplePoint, TrackedArchive, TrackedProcess, TrackingStore,
};
use tauri::{AppHandle, Manager, State, Window};
use tauri_plugin_opener::OpenerExt;

use agent::{AgentState, AgentStatus};
use mcp::{Event, EventBus, McpStatus};

/// The model the agent starts on, matching the chat composer's default picker value.
const DEFAULT_AGENT_MODEL: &str = "qwen";

struct MonitorState(Mutex<sentry_core::Monitor>);
struct HistoryState(Arc<HistoryStore>);
/// `Arc` rather than a bare store because the in-process MCP server shares these exact
/// instances — see [`mcp`]. The UI and an agent must be reading one database, not two
/// handles onto the same file with independent WAL views.
struct ChatState(Arc<ChatStore>);
struct TrackingState(Arc<TrackingStore>);
/// `None` until the MCP server finishes binding in `setup`, and if binding fails it stays
/// `None` — the desktop app is expected to work with no MCP server at all.
struct McpState(Mutex<Option<McpStatus>>);

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
///
/// The work itself lives in [`archive::end_and_archive`] because the MCP server exposes the
/// same operation, and a session ended through one path must be archived exactly like a
/// session ended through the other.
#[tauri::command]
fn end_tracking(
    app: AppHandle,
    tracking: State<TrackingState>,
    history: State<HistoryState>,
    id: i64,
) -> Result<TrackedProcess, String> {
    let data_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    archive::end_and_archive(&tracking.0, &history.0, &data_dir, id)
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
    archive::read_archive(&tracking.0, id)
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

/// Records a user message and tells the agent to answer it.
///
/// The push is the point: without it the agent would have to ask repeatedly whether anything
/// had arrived. Publishing after the write, never before, so the agent cannot read the space
/// before the message is committed.
#[tauri::command]
fn send_chat_message(
    state: State<ChatState>,
    bus: State<EventBus>,
    chat_space_id: i64,
    content: String,
) -> Result<ChatSpaceDetail, String> {
    let detail = state
        .0
        .send_message(chat_space_id, &content)
        .map_err(|e| e.to_string())?;
    bus.publish(&Event::ChatMessage { chat_space_id });
    Ok(detail)
}

/// Terminates a process on behalf of the UI's "Terminate" button.
///
/// `expect_name` should be the name shown on the row the user clicked. Passing it makes the
/// call refuse rather than kill if the pid has been recycled between the last table refresh
/// and the click — see [`sentry_core::Monitor::kill_process`].
///
/// Returns `Ok` with `delivered: false` when the process survives, rather than `Err`: "the
/// process was already gone" is an outcome the UI should report, not an error state.
#[tauri::command]
fn kill_process(
    state: State<MonitorState>,
    pid: u32,
    expect_name: Option<String>,
) -> Result<KillOutcome, String> {
    let mut monitor = state.0.lock().unwrap_or_else(|e| e.into_inner());
    Ok(monitor.kill_process(pid, expect_name.as_deref(), None))
}

/// Where the in-process MCP server is listening, for display in the UI.
#[tauri::command]
fn get_mcp_status(state: State<McpState>) -> Option<McpStatus> {
    state.0.lock().unwrap_or_else(|e| e.into_inner()).clone()
}

#[tauri::command]
fn get_agent_status(state: State<AgentState>) -> AgentStatus {
    state.lock().status()
}

#[tauri::command]
fn start_agent(state: State<AgentState>) -> Result<AgentStatus, String> {
    state.lock().start()
}

#[tauri::command]
fn stop_agent(state: State<AgentState>) -> AgentStatus {
    let mut agent = state.lock();
    agent.stop();
    agent.status()
}

/// Records the model chosen in the chat composer's picker and pushes it to the agent.
///
/// No restart: the running agent swaps in place on receiving the event, so switching models
/// keeps the conversation's history.
#[tauri::command]
fn set_agent_model(state: State<AgentState>, bus: State<EventBus>, model: String) -> AgentStatus {
    let mut agent = state.lock();
    agent.set_model(model.clone());
    bus.publish(&Event::AgentConfig { model });
    agent.status()
}

/// Abandons the agent turn running for `chat_space_id` and closes it out in the transcript.
///
/// Writes the notice here rather than waiting for the agent to confirm: the reason a user
/// reaches for stop is that the agent has stopped responding, so a design that needs the
/// agent to cooperate before the UI unblocks would fail in exactly the case it exists for.
/// The cancel event is published too, so a merely-slow agent drops the work instead of
/// burning tokens on a reply nobody will see.
///
/// Only appends when the space is genuinely still waiting — if a reply landed in the moment
/// between the click and this running, that reply stands and the click is a no-op. The
/// check-and-append is atomic in the store precisely to make that moment as narrow as
/// possible rather than a window this command itself could lose a race in.
#[tauri::command]
fn interrupt_chat(
    state: State<ChatState>,
    bus: State<EventBus>,
    chat_space_id: i64,
) -> Result<Option<ChatSpaceDetail>, String> {
    bus.publish(&Event::CancelChat { chat_space_id });

    state
        .0
        .interrupt_if_awaiting(chat_space_id, "Process interrupted.")
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
        .manage(McpState(Mutex::new(None)))
        .manage(AgentState::new(DEFAULT_AGENT_MODEL))
        .manage(EventBus::new())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let history = Arc::new(HistoryStore::open(data_dir.join("history.sqlite"))?);
            // Runs for the process lifetime, independent of whether the frontend is
            // polling `get_snapshot` — see sentry_core::spawn_recorder.
            let _recorder = sentry_core::spawn_recorder(history.clone(), HISTORY_RECORD_INTERVAL);
            let chat = Arc::new(ChatStore::open(data_dir.join("chat.sqlite"))?);
            let tracking = Arc::new(TrackingStore::open(data_dir.join("tracking.sqlite"))?);

            app.manage(HistoryState(history.clone()));
            app.manage(ChatState(chat.clone()));
            app.manage(TrackingState(tracking.clone()));

            // Host the MCP server in this process, sharing the stores just managed above so
            // an agent and the UI read one database. A failure to bind is logged and
            // swallowed on purpose: the desktop app is fully usable without it, and refusing
            // to start the whole app because a port was taken would be the wrong trade.
            let handle = app.handle().clone();
            let bus = app.state::<EventBus>().inner().clone();
            tauri::async_runtime::spawn(async move {
                match mcp::start(
                    handle.clone(),
                    history,
                    chat,
                    tracking,
                    bus,
                    DEFAULT_AGENT_MODEL.to_string(),
                )
                .await
                {
                    Ok(status) => {
                        println!("sentry-mcp: listening on {}", status.url);
                        let state = handle.state::<McpState>();
                        *state.0.lock().unwrap_or_else(|e| e.into_inner()) = Some(status);

                        // Only now — the agent connects to the MCP server on startup, so
                        // launching it before the listener is bound would just make it fail
                        // its first connection.
                        if let Err(e) = handle.state::<AgentState>().lock().start() {
                            eprintln!("sentry-agent: not started: {e}");
                        }
                    }
                    Err(e) => eprintln!("sentry-mcp: failed to start: {e}"),
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            close_window,
            minimize_window,
            maximize_window,
            get_snapshot,
            get_process_details,
            kill_process,
            get_mcp_status,
            get_agent_status,
            start_agent,
            stop_agent,
            set_agent_model,
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
            interrupt_chat,
            write_and_open_file
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            // Kill the agent when the app goes away. `AgentProcess` also stops itself on
            // drop, but managed state isn't reliably dropped on process exit, and an
            // orphaned agent would sit there polling the database forever.
            if matches!(event, tauri::RunEvent::Exit) {
                app.state::<AgentState>().lock().stop();
            }
        });
}
