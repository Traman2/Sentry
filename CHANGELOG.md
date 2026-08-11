# Changelog

## v0.1.0 — 2026-08-11

Sentry is a desktop system monitor and activity tracker built with Tauri and Rust. This is the initial release.

### sentry-core
- Real-time system monitoring via `sysinfo`: per-process rows (CPU, memory, disk I/O), network interfaces, disk volumes, a whole-machine summary (OS, uptime, aggregate CPU/mem/swap), hardware temperature sensors, and system user accounts.
- Persistent sample history (SQLite-backed): a background recorder samples the system on a fixed interval so CPU/memory/disk-I/O trends can be charted over time, independent of whether a window is open, with 24-hour default retention and automatic pruning.
- "Track" sessions (SQLite-backed): start/end/list/delete a watch over a fixed set of pids (a single process or a whole app); ended sessions archive their samples to a JSON file so they outlive the history retention window.
- Persistent chat spaces (SQLite-backed): conversation threads with an id, a title seeded from the first message, and ordered messages — storage groundwork for the chat panel.

### sentry-tauri-ui
- Resource Monitor tab: sortable/paginated process table, toolbar, multi-select action bar, export-to-CSV modal, and a "view more" process detail modal.
- Track tab: per-session CPU/memory charts over time, a paginated session table, an ended-session banner, and a track header.
- Sidebar panels: an Activity Monitor with live gauges and a Details View for chat/process detail as primary sidebars, plus a collapsible sidebar with a tab bar and a modal socket for app-wide modals.
- Chat: a Chatbot sidebar panel and a ChatSpace tab page backed by the chat store, with the ability to create and delete chat spaces.
- Welcome/empty-state screen, app logo, and a navbar with window chrome.
