# Tab architecture

How the tab bar and tab content area work, kept up to date as the source of truth for
`src/store/tabs.ts`, `src/config/tabs.ts`, `src/components/TabBar.tsx`, and
`src/components/TabContent.tsx`. For the step-by-step of *adding* a new tab type, see the
`component-scaffold` Claude Code skill (`.claude/skills/component-scaffold/SKILL.md`) — this
file documents what exists, that skill documents how to extend it.

## The pieces

- **`store/tabs.ts`** — a Zustand store (`useTabStore`) holding `tabs: Tab[]` and
  `activeTabId: string | null`. A `Tab` is `{ id, type, title }`, where `type` is one of the
  registered tab types (see below). Actions: `openTab` (no-ops into `setActiveTab` if a tab
  with that `id` already exists, otherwise appends and activates it), `closeTab`,
  `discardTab` (like `closeTab`, but also drops the tab from `recent` — use this when the
  underlying resource was deleted, not just closed), `setActiveTab`, `renameTab`. The store
  also tracks `recent: Tab[]` — up to 10 most-recently-opened tabs (excluding Welcome),
  surviving a `closeTab` so Welcome's "Recent" panel can link back to them.
- **`config/tabs.ts`** — the `TABS` registry: one `TabConfig` per tab type
  (`{ type, defaultId, defaultTitle, component, showInViewMenu }`). `TabContent.tsx` builds a
  `type → component` lookup from this array at module load.
- **`components/TabBar.tsx`** — the horizontal strip of open tabs, one per entry in
  `tabs`. Click to activate, the trailing `X` to close (`stopPropagation`, so it doesn't also
  activate). Vertical mouse-wheel scroll is redirected to horizontal, matching VS Code's tab
  strip — trackpad users already send real `deltaX` and are left alone.
- **`components/TabContent.tsx`** — renders **every** open tab simultaneously, each in its own
  `div`, toggling visibility with `display: none`/`block` rather than conditionally mounting.
  This is deliberate: a tab's internal component state (scroll position, an in-flight draft,
  React Query-style local cache) survives switching away and back, because the component is
  never unmounted.
- **`App.tsx`** composes `Navbar` → (`Sidebar` + `SidePanel` + (`AppLogo` + `TabBar` +
  `TabContent`) + `ModalSocket`) as a vertical flex column filling the window.

## Registered tab types

| `type` | Component | Instancing | `showInViewMenu` |
| --- | --- | --- | --- |
| `welcome` | `tab-pages/Welcome.tsx` | Singleton (`defaultId: "welcome"`) | Yes |
| `resource-monitor` | `tab-pages/ResourceMonitor.tsx` | Singleton | Yes |
| `test` | `tab-pages/TestPage.tsx` | Singleton | Yes |
| `chat-space` | `tab-pages/ChatSpace.tsx` | Multi-instance — one tab per chat space, `id` is the space's database id | No |
| `track` | `tab-pages/Track.tsx` | Multi-instance — one tab per tracked session, `id` is `` `track-${session.id}` `` | No |
| `mcp-client` | `tab-pages/McpClientUsage.tsx` | Multi-instance — one tab per MCP client, `id` is the client's database id | No |

**Singleton** tab types open at their fixed `defaultId` from the Navbar's "View" menu
(`showInViewMenu: true`) — there is ever only one tab of that type. **Multi-instance** types
are opened dynamically, each with its own `id` sourced from backend data, typically from a
sidebar panel's "+ New" button and list (`Chatbot.tsx` → `chat-space`, `DetailsView.tsx` →
`track`) or a list-only panel with no create action (`McpClients.tsx` → `mcp-client`, since
clients are discovered, not created by the user). They opt out of the View menu — opening one
at a single fixed id would be meaningless when several can be open with different data.

Every tab page component has the signature `({ tabId }: { tabId: string }) => ReactNode`.
Singleton pages can ignore the prop; multi-instance pages use it to know which instance
they're rendering (e.g. `ChatSpace.tsx` reads `tabId` as the chat space id to fetch/send for
that specific conversation).

## Known gotcha: circular import

`config/tabs.ts` imports every tab page component to build `TABS`. If a tab page component
imports `TABS`, or anything else from `config/tabs.ts` as a *value* (not `import type`), that's
a cycle — `config/tabs.ts` → `YourTab.tsx` → `config/tabs.ts` — which crashes at runtime with
`Cannot access 'TABS' before initialization`. If a tab page needs to open another tab, hardcode
the `Tab` object inline and call `useTabStore().openTab(...)` directly instead of importing
`TABS`. See `Welcome.tsx`'s `RESOURCE_MONITOR_TAB`/`TEST_PAGE_TAB` constants (singleton) or
`Chatbot.tsx`/`ChatSpace.tsx` (multi-instance, id comes from backend data). A type-only import
(`import type { TabType } from "../config/tabs"`) is safe.
