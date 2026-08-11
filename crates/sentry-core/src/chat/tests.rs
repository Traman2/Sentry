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
fn first_message_sets_title_and_records_placeholder_reply() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();

    let detail = store.send_message(space.id, "why is cpu high").unwrap();

    assert_eq!(detail.title, "why is cpu high");
    assert_eq!(detail.messages.len(), 2);
    assert_eq!(detail.messages[0].role, "user");
    assert_eq!(detail.messages[0].content, "why is cpu high");
    assert_eq!(detail.messages[1].role, "assistant");
    assert_eq!(detail.messages[1].content, "You asked \"why is cpu high\"");
}

#[test]
fn second_message_does_not_change_title() {
    let store = ChatStore::open_in_memory().unwrap();
    let space = store.create_chat_space().unwrap();
    store.send_message(space.id, "first").unwrap();

    let detail = store.send_message(space.id, "second").unwrap();

    assert_eq!(detail.title, "first");
    assert_eq!(detail.messages.len(), 4);
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
