use super::TrackingStore;

#[test]
fn start_creates_active_session() {
    let store = TrackingStore::open_in_memory().unwrap();
    let session = store.start("firefox", &[100, 101]).unwrap();

    assert_eq!(session.name, "firefox");
    assert_eq!(session.pids, vec![100, 101]);
    assert_eq!(session.status, "active");
    assert!(session.ended_at_ms.is_none());
    assert!(session.archive_path.is_none());
}

#[test]
fn list_filters_by_name_and_orders_most_recent_first() {
    let store = TrackingStore::open_in_memory().unwrap();
    let a = store.start("firefox", &[1]).unwrap();
    let _b = store.start("chrome", &[2]).unwrap();
    let c = store.start("firefox", &[3]).unwrap();

    let firefox_sessions = store.list(Some("firefox")).unwrap();
    assert_eq!(firefox_sessions.len(), 2);
    assert_eq!(firefox_sessions[0].id, c.id);
    assert_eq!(firefox_sessions[1].id, a.id);

    let all_sessions = store.list(None).unwrap();
    assert_eq!(all_sessions.len(), 3);
}

#[test]
fn end_marks_ended_and_is_idempotent() {
    let store = TrackingStore::open_in_memory().unwrap();
    let session = store.start("firefox", &[100]).unwrap();

    let ended = store.end(session.id).unwrap().unwrap();
    assert_eq!(ended.status, "ended");
    assert!(ended.ended_at_ms.is_some());

    // Ending again doesn't clobber the first ended_at_ms.
    let ended_again = store.end(session.id).unwrap().unwrap();
    assert_eq!(ended_again.ended_at_ms, ended.ended_at_ms);
}

#[test]
fn set_archive_path_updates_the_session() {
    let store = TrackingStore::open_in_memory().unwrap();
    let session = store.start("firefox", &[100]).unwrap();
    store.end(session.id).unwrap();

    let archived = store
        .set_archive_path(session.id, "/data/tracked/1.json")
        .unwrap()
        .unwrap();
    assert_eq!(archived.archive_path, Some("/data/tracked/1.json".to_string()));
}

#[test]
fn delete_removes_the_session_and_returns_it() {
    let store = TrackingStore::open_in_memory().unwrap();
    let session = store.start("firefox", &[100]).unwrap();

    let deleted = store.delete(session.id).unwrap();
    assert_eq!(deleted.unwrap().id, session.id);
    assert!(store.get(session.id).unwrap().is_none());
}

#[test]
fn delete_returns_none_when_missing() {
    let store = TrackingStore::open_in_memory().unwrap();
    assert!(store.delete(999).unwrap().is_none());
}
