use super::ChatStore;

#[test]
fn create_lists_with_default_title() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();
    assert_eq!(space.title, "New Chat");

    let listed = store.list_chat_spaces().unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].id, space.id);
}

#[test]
fn first_message_sets_title_and_records_only_the_user_turn() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();

    let detail = store.send_message(space.id, "why is cpu high").unwrap();

    assert_eq!(detail.title, "why is cpu high");
    // Exactly one message: the reply is the agent's job, via `append_message`.
    assert_eq!(detail.messages.len(), 1);
    assert_eq!(detail.messages[0].role, "user");
    assert_eq!(detail.messages[0].content, "why is cpu high");
}

#[test]
fn second_message_does_not_change_title() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();
    store.send_message(space.id, "first").unwrap();

    let detail = store.send_message(space.id, "second").unwrap();

    assert_eq!(detail.title, "first");
    assert_eq!(detail.messages.len(), 2);
}

#[test]
fn list_orders_most_recently_active_first() {
    let store = ChatStore::open_in_memory().unwrap();
    let first = store.create_chat_space().unwrap();
    let second = store.create_chat_space().unwrap();

    store.send_message(first.id, "ping").unwrap();

    let listed = store.list_chat_spaces().unwrap();
    assert_eq!(listed[0].id, first.id);
    assert_eq!(listed[1].id, second.id);
}

#[test]
fn get_chat_space_returns_none_when_missing() {
    let store = ChatStore::open_in_memory().unwrap();
    assert!(store.get_chat_space(999).unwrap().is_none());
}

#[test]
fn delete_chat_space_removes_it_and_its_messages() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();
    store.send_message(space.id, "hello").unwrap();

    assert!(store.delete_chat_space(space.id).unwrap());
    assert!(store.get_chat_space(space.id).unwrap().is_none());
    assert!(store.list_chat_spaces().unwrap().is_empty());
}

#[test]
fn delete_chat_space_returns_false_when_missing() {
    let store = ChatStore::open_in_memory().unwrap();
    assert!(!store.delete_chat_space(999).unwrap());
}

#[test]
fn append_message_adds_one_turn_with_the_given_role() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();
    store.send_message(space.id, "why is cpu high").unwrap();

    let appended = store
        .append_message(space.id, "assistant", "chrome.exe is at 40%", None)
        .unwrap();

    assert_eq!(appended.role, "assistant");
    assert_eq!(appended.content, "chrome.exe is at 40%");
    assert_eq!(appended.chat_space_id, space.id);

    // The user turn plus ours — exactly one added.
    let detail = store.get_chat_space(space.id).unwrap().unwrap();
    assert_eq!(detail.messages.len(), 2);
    assert_eq!(detail.messages[1].id, appended.id);
    assert_eq!(detail.messages[1].content, "chrome.exe is at 40%");
}

#[test]
fn append_message_leaves_the_title_alone() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();

    store
        .append_message(space.id, "user", "this should not become the title", None)
        .unwrap();

    let detail = store.get_chat_space(space.id).unwrap().unwrap();
    assert_eq!(detail.title, "New Chat");
}

#[test]
fn append_message_bumps_the_space_to_the_front_of_the_list() {
    let store = ChatStore::open_in_memory().unwrap();
    let first = store.create_chat_space().unwrap();
    let second = store.create_chat_space().unwrap();
    store.send_message(first.id, "ping").unwrap();

    // `updated_at_ms` has millisecond resolution and `list_chat_spaces` orders by it with no
    // tiebreaker, so without this the two writes can land in the same millisecond and the
    // resulting order is genuinely undefined rather than wrong.
    std::thread::sleep(std::time::Duration::from_millis(2));
    store
        .append_message(second.id, "assistant", "pong", None)
        .unwrap();

    let listed = store.list_chat_spaces().unwrap();
    assert_eq!(listed[0].id, second.id);
    assert!(listed[0].updated_at_ms > listed[1].updated_at_ms);
}

#[test]
fn append_message_errors_on_a_missing_chat_space() {
    let store = ChatStore::open_in_memory().unwrap();
    assert!(
        store
            .append_message(999, "assistant", "nobody home", None)
            .is_err()
    );
}

#[test]
fn append_message_round_trips_an_error_turn_with_its_details() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();
    store.send_message(space.id, "why is cpu high").unwrap();

    store
        .append_message(
            space.id,
            "error",
            "Groq rate limit exceeded",
            Some("Traceback (most recent call last):\n  ..."),
        )
        .unwrap();

    let detail = store.get_chat_space(space.id).unwrap().unwrap();
    let last = detail.messages.last().unwrap();
    assert_eq!(last.role, "error");
    assert_eq!(last.content, "Groq rate limit exceeded");
    assert!(last.details.as_deref().unwrap().starts_with("Traceback"));
}

#[test]
fn messages_without_details_read_back_as_none() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();

    let detail = store.send_message(space.id, "hello").unwrap();

    assert!(detail.messages[0].details.is_none());
}

#[test]
fn interrupt_if_awaiting_appends_when_the_last_turn_is_a_user_message() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();
    store.send_message(space.id, "why is cpu high").unwrap();

    let detail = store
        .interrupt_if_awaiting(space.id, "Process interrupted.")
        .unwrap()
        .expect("a user turn was awaiting a reply");

    assert_eq!(detail.messages.len(), 2);
    assert_eq!(detail.messages[1].role, "interrupted");
    assert_eq!(detail.messages[1].content, "Process interrupted.");
}

#[test]
fn interrupt_if_awaiting_is_a_no_op_once_a_reply_has_landed() {
    // Regression test for a race between the interrupt command and the agent's own reply: if
    // the reply has already landed — whether before the click or in the gap a two-step
    // check-then-append would have left open — interrupting must not overwrite it.
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();
    store.send_message(space.id, "why is cpu high").unwrap();
    store
        .append_message(space.id, "assistant", "chrome.exe is at 40%", None)
        .unwrap();

    let result = store
        .interrupt_if_awaiting(space.id, "Process interrupted.")
        .unwrap();

    assert!(result.is_none());
    let detail = store.get_chat_space(space.id).unwrap().unwrap();
    assert_eq!(detail.messages.len(), 2);
    assert_eq!(detail.messages[1].role, "assistant");
}

#[test]
fn interrupt_if_awaiting_returns_none_for_a_missing_chat_space() {
    let store = ChatStore::open_in_memory().unwrap();
    assert!(
        store
            .interrupt_if_awaiting(999, "Process interrupted.")
            .unwrap()
            .is_none()
    );
}

#[test]
fn opening_a_database_twice_does_not_re_add_the_details_column() {
    // The details column is added by ALTER TABLE, which errors if run twice — so opening an
    // existing database has to detect that it's already there. An in-memory database is
    // discarded on close, so this uses a real file to get a genuine second open.
    let dir = std::env::temp_dir().join(format!("system-expert-chat-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("chat.sqlite");
    let _ = std::fs::remove_file(&path);

    let first = ChatStore::open(&path).unwrap();
    let space = first.create_chat_space().unwrap();
    first
        .append_message(space.id, "error", "boom", Some("trace"))
        .unwrap();
    drop(first);

    let reopened = ChatStore::open(&path).expect("second open should succeed");
    let detail = reopened.get_chat_space(space.id).unwrap().unwrap();
    assert_eq!(detail.messages[0].details.as_deref(), Some("trace"));

    let _ = std::fs::remove_file(&path);
}
