Build the tab management system for my Tauri + React + TypeScript + Tailwind
+ shadcn/ui app called Sentry (a system process monitor).

Create these three files:

1. src/store/tabs.ts
   - Use zustand (already installed) to create a `useTabStore` hook
   - Tab shape: { id: string, type: 'table' | 'process-detail' | 'chat',
     title: string, meta?: Record<string, unknown> }
   - Store state: tabs: Tab[], activeTabId: string | null
   - Actions:
     - openTab(tab: Tab) — if a tab with that id already exists, just set it
       active instead of duplicating; otherwise append it and set it active
     - closeTab(id: string) — remove the tab; if the closed tab was active,
       set the new active tab to the last remaining tab (or null if none left)
     - setActiveTab(id: string)
   - Initialize with one default tab: { id: 'main-table', type: 'table',
     title: 'Processes' }, active by default

2. src/components/TabBar.tsx
   - Reads tabs/activeTabId from useTabStore
   - Renders each tab as a clickable pill/button, active tab visually
     distinct (different background + font weight)
   - Each tab (except when only one tab is open) has a small close (X)
     button using lucide-react, calls closeTab, and stopPropagation so
     clicking X doesn't also select the tab
   - Style: h-9, thin bottom border, background #f5f2ea to match my
     existing titlebar, active tab background white
   - This bar sits directly below my existing Navbar component (which
     handles window drag/minimize/maximize/close — don't touch Navbar)

3. src/components/TabContent.tsx
   - Reads tabs/activeTabId from useTabStore
   - Renders ALL open tabs simultaneously in a relative positioned
     container, each in an absolute inset-0 div
   - Use `style={{ display: tab.id === activeTabId ? 'block' : 'none' }}`
     — do NOT conditionally render/unmount inactive tabs, they must stay
     mounted to preserve their internal state when switching
   - For now, since the actual view components (ProcessTable,
     ProcessDetail, ChatView) don't exist yet, render a placeholder div
     for each tab type showing `Placeholder: {tab.type} — {tab.title}`
     so the structure is testable before I build the real content

Also show me how to compose Navbar + TabBar + TabContent together in
App.tsx as a vertical flex column filling the window.

Don't build ProcessTable, ProcessDetail, or ChatView yet — just the
store and the tab shell with placeholders. I'll build those next as a
separate step.