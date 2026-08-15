# Changelog

## v1.0.0 — 2026-08-14

System-Expert's first stable release turns it from a system monitor into an AI-operated one: an in-app chat panel lets a language-model agent inspect and act on your machine through a full MCP tool surface, with a new usage audit trail showing exactly who called what.

### system-expert-core
- New chat storage: persistent chat spaces with full message history (user/assistant/error turns), so conversations with the agent survive restarts.
- New MCP client identification: every tool call is attributed to a specific caller (Claude Code, Claude Desktop, the bundled Python agent, ChatGPT, Grok, or unknown), resolved from both the MCP handshake and the OS process behind the connection.
- New loopback connection resolution: matches an inbound MCP request to the actual OS process that made it, by inspecting the machine's own TCP table.

### system-expert-tauri-ui / system-expert-mcp (in-process MCP server)
- The desktop app now hosts a full MCP server (Streamable HTTP over loopback) exposing 22 tools: browsing/searching processes and their details, killing a process safely (name-verified, with protection for critical OS processes), reading live system/disk/network summaries, querying historical CPU/memory timelines, starting/stopping/archiving tracked monitoring sessions, and reading/writing the chat panel (including posting replies, thinking-step progress, and error reports).
- Every tool call is now logged with its caller, arguments, status, and duration, feeding a new MCP client usage audit page.
- The app now launches and supervises a bundled Python AI agent process automatically — no manual setup required — and tracking sessions can be ended and archived to disk so their history isn't lost to normal data retention.

### system-expert-tauri-ui frontend
- New Chat tab: a full conversational UI for talking to the agent, with live streamed progress ("Calling get_system_summary", etc.), markdown rendering, and error display.
- New MCP Clients sidebar and usage page: shows every connected AI client, its call history, and per-call status/timing — a transparency/audit view into what an agent has been doing to the machine.
- Tracking (Track tab) gained richer session detail: stat cards, metric toolbar, pagination, and an ended-session banner tied to the new archival feature.

### python-agent (new)
- A standalone LangGraph-based agent package that connects to the app's MCP server, answers chat messages using a selectable LLM (GPT-OSS 120B/20B, Qwen, or Llama, served via Groq), and streams its tool-use progress back into the chat panel in real time.
- Supports hot-swapping models mid-conversation from the app's UI without losing chat history, and automatically reconnects and catches up on missed messages if the app restarts.

## v0.1.0 — 2026-08-11

System-Expert is a desktop system monitor and activity tracker built with Tauri and Rust. This is the initial release.

### system-expert-core
- Real-time system monitoring via `sysinfo`: per-process rows (CPU, memory, disk I/O), network interfaces, disk volumes, a whole-machine summary (OS, uptime, aggregate CPU/mem/swap), hardware temperature sensors, and system user accounts.
- Persistent sample history (SQLite-backed): a background recorder samples the system on a fixed interval so CPU/memory/disk-I/O trends can be charted over time, independent of whether a window is open, with 24-hour default retention and automatic pruning.
- "Track" sessions (SQLite-backed): start/end/list/delete a watch over a fixed set of pids (a single process or a whole app); ended sessions archive their samples to a JSON file so they outlive the history retention window.
- Persistent chat spaces (SQLite-backed): conversation threads with an id, a title seeded from the first message, and ordered messages — storage groundwork for the chat panel.

### system-expert-tauri-ui
- Resource Monitor tab: sortable/paginated process table, toolbar, multi-select action bar, export-to-CSV modal, and a "view more" process detail modal.
- Track tab: per-session CPU/memory charts over time, a paginated session table, an ended-session banner, and a track header.
- Sidebar panels: an Activity Monitor with live gauges and a Details View for chat/process detail as primary sidebars, plus a collapsible sidebar with a tab bar and a modal socket for app-wide modals.
- Chat: a Chatbot sidebar panel and a ChatSpace tab page backed by the chat store, with the ability to create and delete chat spaces.
- Welcome/empty-state screen, app logo, and a navbar with window chrome.
