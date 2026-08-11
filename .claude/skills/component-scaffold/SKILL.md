---
name: component-scaffold
description: Scaffold a new modal, sidebar panel, tab page, or shared UI component for the sentry-tauri-ui desktop app, following this repo's established file placement, state, and styling conventions. Use whenever the user asks to add a new modal, a new sidebar panel, a new tab, or a new reusable component to crates/sentry-tauri-ui — before writing any code from scratch.
---

# Component Scaffold

This skill packages the conventions this codebase already settled on, so a
new component matches the existing ones without re-deriving the patterns by
reading around the repo. Everything you need to scaffold correctly is
below — you should not need to grep the frontend for "how do we usually do
X" before using this skill.

All paths are relative to `crates/sentry-tauri-ui/src/`.

## Step 0: pick the archetype

| The user wants... | Archetype | Lives in |
| --- | --- | --- |
| A popup/dialog opened on top of the app | **Modal** | `modals/` |
| New content in one of the 3 primary-sidebar buttons | **Sidebar panel** | `primary-sidebars/` |
| A new top-level tab (shown in the tab bar) | **Tab page** | `tab-pages/` |
| A reusable widget used by 2+ features (not a modal/panel/tab itself) | **Shared component** | `components/` |
| Pure logic/formatting with no JSX | Not a component — goes in `lib/` | `lib/` |

If ambiguous, ask the user which one before scaffolding.

## Global conventions (apply to every archetype)

- **Path alias**: use `@/` for cross-directory imports (`@/components/ui/button`, `@/lib/utils`), not long `../../` chains, for anything outside the immediate folder.
- **State**: if the component needs shared/global state, add a Zustand store in `store/`, following the existing shape exactly — plain `create<T>((set, get) => ({...}))`, no middleware, interface declares state + actions together, consumed via inline selectors (`useXStore((state) => state.foo)`). See `store/modal.ts` (smallest) or `store/tabs.ts` (has actions) as the canonical examples. For a store whose actions call Tauri commands (`invoke(...)`) and sync the result back into state — including one action composing another via `get()` (e.g. `createChatSpaceWithMessage` calling `get().createChatSpace()` then `get().sendMessage()`) — see `store/chat.ts`.
- **Tauri command args**: `invoke("some_command", { fooBar: x })` — the JS object keys are camelCase even though the `#[tauri::command]` fn's Rust parameters are snake_case (`foo_bar: X`); Tauri converts automatically. Don't snake_case the JS side to "match" Rust.
- **Styling primitives**: never hand-roll a button, dropdown, menubar, or dialog — use the wrapped `@base-ui/react` primitives in `components/ui/` (`button.tsx`, `dropdown-menu.tsx`, `menubar.tsx`, `dialog.tsx`). If you need a new primitive not yet wrapped, follow the same wrap-with-`cva`-and-`cn` pattern as those files, don't import `@base-ui/react` raw into a feature component.
- **Design tokens**: see the `ui-consistency-check` skill for the full palette/spacing/typography rules — apply them as you write, don't scaffold first and fix styling later.
- **Icons**: `lucide-react`, typically `h-3.5 w-3.5` or `h-4 w-4`, colored `text-navy/40` to `text-navy/60` when muted/decorative.

## Archetype: Modal

Reference implementation: `modals/ViewMoreModal.tsx` (rich, scrollable) or `modals/ExportSummaryModal.tsx` (simple).

1. Create `modals/<Name>.tsx`. It receives whatever props it needs as a normal component — **no special modal props**, it's just a component that happens to render inside the modal socket.
2. **The modal socket owns positioning/dimming only. Your component owns its entire visual card** — border, rounded corners, background, padding, shadow. Never assume the socket gives you a card.
3. Skeleton for anything with a header/footer (use this instead of a single flat `p-6` div once there's more than ~3 pieces of content, since a fixed-position header/footer with a scrollable middle is the established pattern):
   ```tsx
   <div className="flex max-h-[80vh] w-160 flex-col overflow-hidden rounded-xl border border-teal bg-canvas shadow-xl">
     <div className="flex shrink-0 items-start justify-between border-b border-teal/50 px-6 py-6">
       {/* title via DialogTitle, close (X) via DialogClose */}
     </div>
     <div className="flex-1 overflow-y-auto px-6 py-6">
       {/* scrollable body */}
     </div>
     <div className="flex shrink-0 justify-end border-t border-teal/50 p-6">
       {/* actions via DialogClose / buttons */}
     </div>
   </div>
   ```
   Import `DialogTitle`, `DialogClose` from `@/components/ui/dialog` for the title/close button — don't use raw `<h2>`/`<button>` for those two, they need the base-ui Dialog context for a11y and click-to-close.
4. **To open it**, the caller does `useModalStore().openModal(<YourModal {...props} />)` — element-based, not a registry id. Import the modal component directly at the call site.
5. If the modal needs data captured **at click time** (a frozen snapshot, not live-updating), pass it as a prop from the caller's current render — don't have the modal subscribe to a live store/poll itself unless "live-updating while open" is actually the intent.

## Archetype: Sidebar panel

Reference: `primary-sidebars/ActivityMonitor.tsx` and `primary-sidebars/Chatbot.tsx` (a "+ New" action plus a live list rendered from a store, each item opening a tab — a good reference for any panel that's a create-and-list UI in front of a tab archetype).

1. Create `primary-sidebars/<Name>.tsx`. No special props — `SidePanel.tsx` renders it as `<PanelContent />` with zero arguments.
2. Register it in `config/panels.ts`: add an entry to the `PANELS` array with `{ id, icon, label, component }`. `id` must also be added to `PanelConfig["id"]`'s union type in that same file.
3. The panel gets `p-3` from its `SidePanel.tsx` wrapper already — don't re-add outer padding, just build the content.
4. If it needs live system data, reuse `useSystemSnapshot` and `groupProcessesByApp` from `@/tab-pages/ResourceMonitor/...` rather than re-fetching — see `ActivityMonitor.tsx`.

## Archetype: Tab page

Reference: `tab-pages/Welcome.tsx` (simple) and `tab-pages/ResourceMonitor.tsx` (complex, with its own subfolder).

1. Create `tab-pages/<Name>.tsx` (or a `tab-pages/<Name>/` folder with an index page + subcomponents if it's complex, like `ResourceMonitor/`).
2. Register it in `config/tabs.ts`: add `{ type, defaultId, defaultTitle, component, showInViewMenu }` to `TABS`, and add the new `type` string to `TabConfig["type"]`'s union.
3. Every tab page component has the signature `({ tabId }: { tabId: string }) => ReactNode` — `TabContent.tsx` passes each rendered tab's own `id` in as `tabId`. A **singleton** tab page (one fixed instance — Welcome, ResourceMonitor, TestPage) can just ignore the prop (a zero-arg function component satisfies the type fine). A **multi-instance** tab page — several open tabs sharing the same `type` but each showing different data, like chat spaces — must read `tabId` to know which instance it's rendering; see `ChatSpace.tsx`, which uses `tabId` as the chat space's id to fetch/send for that specific conversation.
4. `showInViewMenu: boolean` controls whether the Navbar's "View" menu lists this tab type (it opens tabs at the type's single fixed `defaultId`, which only makes sense for singleton tabs). Set `false` for multi-instance types — they're opened dynamically instead, each with its own id, typically from a sidebar panel's "+ New" button and list (see `Chatbot.tsx` opening `{ id: String(space.id), type: "chat-space", title: space.title }`).
5. **Critical gotcha — circular import.** `config/tabs.ts` imports every tab page component (to build `TABS`). If a tab page component itself imports `TABS` or anything from `config/tabs.ts` as a *value* (not `import type`), you get a circular dependency: `config/tabs.ts` → `YourTab.tsx` → `config/tabs.ts`, which crashes at runtime with `Cannot access 'TABS' before initialization`. If your tab page needs to open another tab, **do not** import `TABS`/`config/tabs` — instead hardcode a `Tab` object (`{ id, type, title }`) inline and call `useTabStore().openTab(thatObject)` directly. See `Welcome.tsx`'s `RESOURCE_MONITOR_TAB`/`TEST_PAGE_TAB` constants for the singleton pattern, or `ChatSpace.tsx`/`Chatbot.tsx` for the multi-instance pattern (id comes from backend data, not a constant). A type-only import (`import type { TabType } from "../config/tabs"`) is safe and doesn't cause the cycle.
6. Every tab page's root element should be `flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm` — this is what makes every tab look consistent inside `TabContent`.
7. If a tab's title should change after the fact (e.g. a chat space's title is set from the user's first message, decided after the tab is already open), use `useTabStore().renameTab(id, title)` — don't try to mutate the `Tab` object directly.

## Archetype: Shared component

Reference: `components/Gauge.tsx` (tiny, pure-presentational, takes props) and `components/AppLogo.tsx` (reads from a store itself).

1. Create `components/<Name>.tsx`. Keep it prop-driven where possible (like `Gauge`) rather than reaching into stores itself, unless it's genuinely app-wide singleton UI (like `AppLogo`, `Navbar`, `ModalSocket`, which do read stores directly because there's only ever one instance).
2. If it's extracting duplicated logic/markup you're about to copy-paste a second time (e.g. the CPU gauge ring existed in both the table and the sidebar), that's exactly when it belongs here — see how `Gauge.tsx` + `lib/health.ts` got factored out of `columns.tsx` for the second use in `ActivityMonitor.tsx`.

## After scaffolding

Run `npx tsc --noEmit -p .` from `crates/sentry-tauri-ui/` to catch wiring mistakes (missing registration, wrong prop types, the circular-import gotcha above) before considering the component done.
