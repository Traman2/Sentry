use super::identify::ClientIdentity;
use super::store::CallStatus;
use super::McpUsageStore;

fn claude_code() -> ClientIdentity {
    ClientIdentity {
        protocol_name: Some("claude-code".to_string()),
        protocol_version: Some("1.0.0".to_string()),
        pid: Some(1234),
        process_name: Some("claude.exe".to_string()),
        process_command: None,
    }
}

fn python_agent() -> ClientIdentity {
    ClientIdentity {
        protocol_name: Some("mcp".to_string()),
        protocol_version: Some("1.0.0".to_string()),
        pid: Some(5678),
        process_name: Some("python.exe".to_string()),
        process_command: Some("python.exe app.py --serve --model qwen".to_string()),
    }
}

#[test]
fn record_call_creates_a_new_client_on_first_call() {
    let store = McpUsageStore::open_in_memory().unwrap();
    let (client, call) = store
        .record_call(&claude_code(), "list_processes", Some("{}"), CallStatus::Ok, 12, None)
        .unwrap();

    assert_eq!(client.key, "claude-code");
    assert_eq!(client.kind, "claude-code");
    assert_eq!(client.call_count, 1);
    assert_eq!(call.tool_name, "list_processes");
    assert_eq!(call.client_id, client.id);
    assert_eq!(call.status, "ok");
}

#[test]
fn repeated_calls_from_the_same_client_reuse_its_row_and_bump_call_count() {
    let store = McpUsageStore::open_in_memory().unwrap();
    let (first, _) = store
        .record_call(&claude_code(), "list_processes", None, CallStatus::Ok, 10, None)
        .unwrap();
    let (second, _) = store
        .record_call(&claude_code(), "get_system_summary", None, CallStatus::Ok, 8, None)
        .unwrap();

    assert_eq!(first.id, second.id);
    assert_eq!(second.call_count, 2);

    let clients = store.list_clients().unwrap();
    assert_eq!(clients.len(), 1);
}

#[test]
fn distinct_clients_get_distinct_rows() {
    let store = McpUsageStore::open_in_memory().unwrap();
    store
        .record_call(&claude_code(), "list_processes", None, CallStatus::Ok, 10, None)
        .unwrap();
    store
        .record_call(&python_agent(), "get_system_summary", None, CallStatus::Ok, 10, None)
        .unwrap();

    let clients = store.list_clients().unwrap();
    assert_eq!(clients.len(), 2);
    assert!(clients.iter().any(|c| c.kind == "claude-code"));
    assert!(clients.iter().any(|c| c.kind == "python-agent"));
}

#[test]
fn list_clients_orders_most_recently_active_first() {
    let store = McpUsageStore::open_in_memory().unwrap();
    let (first, _) = store
        .record_call(&claude_code(), "list_processes", None, CallStatus::Ok, 10, None)
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    let (second, _) = store
        .record_call(&python_agent(), "get_system_summary", None, CallStatus::Ok, 10, None)
        .unwrap();

    let listed = store.list_clients().unwrap();
    assert_eq!(listed[0].id, second.id);
    assert_eq!(listed[1].id, first.id);
}

#[test]
fn get_client_usage_returns_calls_newest_first() {
    let store = McpUsageStore::open_in_memory().unwrap();
    let (client, _) = store
        .record_call(&claude_code(), "list_processes", None, CallStatus::Ok, 10, None)
        .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(2));
    store
        .record_call(&claude_code(), "kill_process", None, CallStatus::Ok, 5, None)
        .unwrap();

    let detail = store.get_client_usage(client.id).unwrap().unwrap();
    assert_eq!(detail.calls.len(), 2);
    assert_eq!(detail.calls[0].tool_name, "kill_process");
    assert_eq!(detail.calls[1].tool_name, "list_processes");
    assert_eq!(detail.call_count, 2);
}

#[test]
fn delete_client_removes_it_and_its_calls() {
    let store = McpUsageStore::open_in_memory().unwrap();
    let (client, _) = store
        .record_call(&claude_code(), "list_processes", None, CallStatus::Ok, 10, None)
        .unwrap();

    assert!(store.delete_client(client.id).unwrap());
    assert!(store.get_client_usage(client.id).unwrap().is_none());
    assert!(store.list_clients().unwrap().is_empty());
}

#[test]
fn delete_client_returns_false_when_missing() {
    let store = McpUsageStore::open_in_memory().unwrap();
    assert!(!store.delete_client(999).unwrap());
}

#[test]
fn get_client_usage_returns_none_when_missing() {
    let store = McpUsageStore::open_in_memory().unwrap();
    assert!(store.get_client_usage(999).unwrap().is_none());
}

#[test]
fn protocol_error_records_the_error_message() {
    let store = McpUsageStore::open_in_memory().unwrap();
    let (client, call) = store
        .record_call(
            &claude_code(),
            "no_such_tool",
            None,
            CallStatus::ProtocolError,
            1,
            Some("method not found"),
        )
        .unwrap();

    let detail = store.get_client_usage(client.id).unwrap().unwrap();
    assert_eq!(detail.calls[0].status, "protocol_error");
    assert_eq!(detail.calls[0].error_message.as_deref(), Some("method not found"));
    assert_eq!(call.status, "protocol_error");
}

#[test]
fn params_json_longer_than_the_cap_is_truncated() {
    let store = McpUsageStore::open_in_memory().unwrap();
    let huge = "x".repeat(super::store::MAX_PARAMS_JSON_LEN + 500);
    let (_, call) = store
        .record_call(&claude_code(), "list_processes", Some(&huge), CallStatus::Ok, 1, None)
        .unwrap();

    let stored = call.params_json.unwrap();
    assert!(stored.len() <= super::store::MAX_PARAMS_JSON_LEN + '…'.len_utf8());
    assert!(stored.ends_with('…'));
}
