---
name: ui-consistency-check
description: Review new or changed frontend code in crates/sentry-tauri-ui against this app's modern-B2B-SaaS design language, design tokens, spacing, and component-reuse conventions, and report deviations. Use after writing or editing any .tsx/.css in crates/sentry-tauri-ui, or whenever the user asks whether the UI is "on brand", "consistent", "modern", or "flush with the rest of the app".
---

# UI Consistency Check

This skill exists so reviewing UI work doesn't require re-reading the whole
frontend to remember what "normal" looks like — the rules below **are**
that context, extracted from `index.css` and the existing components.
Check changed code against this list directly; only open other files when
you need to confirm how a specific existing pattern is actually used.

Scope: `crates/sentry-tauri-ui/src/`.

## 0. The design language

Sentry's UI targets **modern B2B SaaS** — the Vercel / Linear / Stripe
dashboard idiom, rendered in this app's mint-and-navy brand palette rather
than their grays. When a judgement call isn't settled by the rules below,
ask "how would Vercel's dashboard do this?" and follow that.

What that actually means in practice:

- **Restraint over decoration.** One hairline border, no shadow stacking, no
  gradients outside chart fills. If a divider and a background change both
  separate the same two things, drop one.
- **Data is the hero.** The number is the biggest thing in a stat card; the
  label is small, muted, and above it. Chrome recedes so figures read fast.
- **Say it once.** A panel title, a section label, a card description, and a
  helper line all explaining the same table is three too many. Prefer a bare
  title. Never write a `CardDescription` restating what the title said.
  Never add a banner for a state a badge already shows.
- **Quiet labels, comfortable rows.** Column headers and section labels are
  small and muted, not loud. Table rows get real vertical padding (`py-2.5`
  to `py-3`) — cramped rows are the single most common thing that makes this
  app look homemade.
- **Every state is designed.** Loading gets `Skeleton` in the shape of the
  real content, empty gets `Empty`, not a bare `<p>` or a spinner.
- **Interactive affordances are subtle but present.** Hover fills are light
  (`hover:bg-teal/10`–`/15`), row actions fade in on `group-hover`, and
  everything focusable has a visible `focus-visible` state.

## 1. Color tokens

Source of truth: `crates/sentry-tauri-ui/src/index.css`. These are the only
colors that should appear in component code:

| Token | Value | Use for |
| --- | --- | --- |
| `canvas` | `#ffffff` | surfaces/backgrounds (`bg-canvas`) |
| `mint` | `#dbe8c8` | rarely used directly; backs `--muted` |
| `teal` | `#93c3b8` | borders, dividers, hover fills, secondary/accent (`border-teal`, `bg-teal/10`–`/40`, `text-teal`) |
| `navy` | `#26264f` | text, icons, primary actions (`text-navy`, `bg-navy` for primary buttons) |
| `danger` | `#dc2626` | destructive-only (kill/delete actions, critical load) |
| `warning` | `#c9a227` | mid-threshold load, non-blocking caution |
| `success` | `#3fa66b` | healthy load, "Live"/active status |

Semantic Tailwind classes (`bg-primary`, `text-muted-foreground`,
`bg-accent`, `border-border`, etc.) already resolve to these via the
`:root` block in `index.css` — prefer them for shadcn/base-ui primitives
(`components/ui/*`), but plain feature code overwhelmingly uses the brand
names directly (`text-navy`, `border-teal/50`). Match whichever convention
the file you're editing already uses; don't mix both in one component.

**Flag**: any `#RRGGBB`, `rgb(...)`, or named CSS color (`text-emerald-600`,
`bg-amber-50`) in a `className` or inline `style`. Tailwind's stock palette
is *not* this app's palette — reach for `text-success`/`text-warning`
instead. The one sanctioned exception is `rgba()` built directly off
`--color-navy`'s `38,38,79`, as used for `bg-dot-pattern` and the scrollbar.

**Health/status color** (CPU/memory ring, load bars): don't invent a new
threshold scheme — reuse `healthColor()` from `@/lib/health.ts` (>=50% →
danger, >=15% → warning, else success) and the `Gauge` component
(`@/components/Gauge.tsx`) that renders it as a ring. If you're about to
write a `<circle>`/`<svg>` gauge or a percent-to-color function inline,
stop — import these instead.

**Chart series colors** are the documented exception to "no raw hex": the
four constants in `tab-pages/Track/constants.ts` (`CPU_COLOR`,
`MEMORY_COLOR`, `DISK_READ_COLOR`, `DISK_WRITE_COLOR`) are passed to Recharts
and to legend/accent dots via inline `style`. Add new series colors there,
never inline at the call site.

## 2. Typography scale

| Role | Classes | Example |
| --- | --- | --- |
| Table column header | `text-xs font-medium text-navy/60` | every `TableHead` |
| Section / eyebrow label | `text-[10px] font-semibold uppercase tracking-wide text-navy/50`–`/60` | sidebar section labels, stat card labels |
| Primary body text | `text-xs font-medium text-navy` (or `text-sm` for modal-scale content) | row titles, values |
| Secondary/meta text | `text-[11px] text-muted-foreground` | subtitles, "PID 1234", captions |
| Card / panel title | `text-sm font-medium` via `CardTitle` | chart and table panel headers |
| Stat card value | `font-heading text-xl font-semibold text-navy tabular-nums` | the number in a stat tile |
| Modal/page heading | `text-lg`–`text-xl font-semibold` via `DialogTitle`, or `text-4xl font-bold` for tab-page hero headings (`Welcome.tsx`) | modal titles, page titles |

Column headers are **sentence case, not uppercase** — that's the Vercel
table idiom and it's deliberate. The tiny uppercase treatment is reserved
for *section* labels (a sidebar block, a stat tile's caption), where it
reads as an eyebrow rather than a table header.

Any number that updates live or sits in a column gets `tabular-nums`, so
digits don't jitter as values change.

**Flag**: ad-hoc font sizes outside `text-[10px]`, `text-[11px]`, `text-xs`,
`text-sm`, `text-lg`, `text-xl`, the heading sizes; arbitrary values like
`text-[13px]` invented for one component; uppercase column headers.

## 3. Spacing & sizing

- Tailwind's default 4px scale throughout (`gap-1.5`, `px-3`, `py-2.5`,
  `p-6`) — no arbitrary pixel spacing (`p-[13px]`) unless matching a hard
  external constraint (icon size, scrollbar width compensation).
- **Table density**: header cells `h-9 px-4`; body cells `px-4 py-2.5`
  (dense lists like the process table) to `px-4 py-3` (roomier history
  tables). Never `py-1.5` — that was the old cramped look.
- Card/box padding: `p-3` (compact, sidebar cards) to `p-6` (modal
  sections). `p-4` for mid-weight boxes and chart bodies.
- Panel section spacing: `gap-4` between blocks in a side panel, `gap-px`
  between rows in a list, `gap-2` between stacked cards.
- Fixed component widths use the same 4px-multiple scale as existing
  modals: `w-96` (small modal), `w-160` (large modal, = 640px),
  `w-125`/`h-125` (500px, Welcome page box), `w-64` (side panel rail).
  Reuse these rather than inventing a new width for a similarly-scoped
  component.
- Rounded corners: `rounded-md` (buttons, small controls), `rounded-lg`
  (cards, table panels, tab-page root, list rows), `rounded-xl` (modal card
  root). shadcn `Card` defaults to `rounded-xl` — override it to
  `rounded-lg` outside modals.

## 4. Borders, surfaces, elevation

- Card/box border: `border border-teal/50` (subtle) or `border border-teal`
  (stronger, e.g. tab-page root, modal root). shadcn `Card` uses a ring
  instead — `ring-teal/50`.
- Card fill: `bg-canvas` (default) or `bg-teal/5`–`/10` (subtly tinted).
- Table header fill: `bg-muted` — **opaque**, because sticky headers scroll
  rows underneath themselves and a translucent `bg-teal/10` lets them show
  through.
- Row hover: `hover:bg-teal/10` (table rows) or `hover:bg-teal/15` (sidebar
  list rows). `hover:bg-teal/25` for buttons, menu items, toolbar controls.
- Row divider: `border-b border-teal/25` on the cells.
- Shadow: `shadow-sm` (tab-page root, side panel), `shadow-lg`/`shadow-xl`
  (modals — the more "on top of everything" a surface is, the heavier the
  shadow). Don't add hover shadows to static cards.
- Dividers inside a box: `border-b border-teal/30` between rows,
  `border-b border-teal/40`–`/50` for a header/footer separator.

**Flag**: a new bordered box that doesn't use `border-teal` at some
opacity, a hover state that isn't `teal`-based, or a translucent background
on a `sticky` element.

## 5. Component reuse checklist

Before writing new markup, check whether it already exists. Everything in
`components/ui/` is shadcn (style `base-nova`, built on `@base-ui/react`) —
add more with `npx shadcn@latest add @shadcn/<name>` from
`crates/sentry-tauri-ui`, don't hand-roll.

| Need | Use |
| --- | --- |
| Button | `ui/button` — variants `default`/`outline`/`secondary`/`ghost`/`destructive`/`link`, sizes `xs`/`sm`/`default`/`lg`/`icon*` |
| Paired buttons (pagination) | `ui/button-group` |
| Table | `ui/table` — pass `containerClassName` to control the scroll region |
| Switch between views | `ui/tabs` (not a Menubar dropdown) |
| Pick one of N options | `ui/toggle-group` with `spacing={0}` for a segmented control |
| Status pill | `ui/badge` |
| Callout | `ui/alert` |
| Empty state | `ui/empty`, or `PanelEmpty` from `@/components/PanelList` in a side panel |
| Loading state | `ui/skeleton`, shaped like the real content |
| Sidebar list row | `PanelSection` / `PanelListItem` from `@/components/PanelList` |
| Column sort affordance | `SortIcon` from `@/components/SortIcon` |
| Dropdown / menu | `ui/dropdown-menu`, `ui/menubar` |
| Dialog / modal chrome | `ui/dialog`, inside a component in `modals/`, opened via `useModalStore().openModal(...)` |
| Charts | `ui/chart` + Recharts, via `tab-pages/Track/MetricChart.tsx` |
| CPU/percent ring | `@/components/Gauge.tsx` |
| Byte/rate/percent formatting | `tab-pages/ResourceMonitor/format.ts` (`formatBytes`, `formatRate`, `formatPercent`) |
| Relative timestamps | `@/lib/relativeTime.ts` (`formatRelativeTime`) |
| Process/app aggregation | `groupProcessesByApp`, `findAppRowByPid` in `tab-pages/ResourceMonitor/groupProcesses.ts` |
| Conditional classNames | `cn()` from `@/lib/utils` |

Never a raw styled `<button>` for anything that reads as a primary or
secondary action — raw `<button>` is only for lightweight inline controls
(table sort toggle, a row's delete icon, close icon) that don't fit the
Button's chrome.

**Flag**: an inline SVG gauge, a hand-rolled byte formatter, a hand-rolled
list row, a second zustand-store-less local implementation of something a
store already does (esp. tabs/modal/panel open-close state), or a raw
`@base-ui/react` import inside a feature file instead of the wrapped
`components/ui/` version.

## 6. Layout patterns worth matching exactly

- **Scrollable region inside a fixed-height container** (modals, panels,
  table panels): pinned header/footer (`shrink-0`) + `min-h-0 flex-1
  overflow-y-auto` middle. The `min-h-0` is load-bearing — without it a flex
  child refuses to shrink below its content and the scroll never engages.
  Don't use `h-full` + percentage heights nested inside flex (this exact bug
  happened once already in `ViewMoreModal.tsx`'s history).
- **Data table panel**: the table owns the leftover height and scrolls
  internally, with a sticky header and pagination pinned in the card footer.
  Pagination must never be a thing you scroll a long page to reach. Give the
  panel a floor (`min-h-96`) so it can't collapse on a short window.
- **Tab page root**: always
  `flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm`.
- **Side panel**: `SidePanel` owns the scroll (`min-h-0 flex-1
  overflow-y-auto p-3`); panel contents lay themselves out and must not
  open a second scroll region inside it.
- **Never import a tab page's own config as a value from within that tab
  page** (circular import — see `component-scaffold` for the full
  explanation and the workaround).

## 7. Known traps

Things that have already cost a debugging round here. Check these
specifically when the symptom matches.

- **shadcn `Card` adds a gap you didn't ask for.** `Card` carries
  `gap-(--card-spacing)` (16px) between header, content, and footer, *and*
  `CardHeader` applies its own bottom padding via
  `[.border-b]:pb-(--card-spacing)`, which compiles to a self-compound
  selector at specificity 0-2-0 — a plain `py-*` override loses to it. For a
  flush table panel, put `gap-0 py-0` on the `Card` and pad each section
  explicitly.
- **Sticky table headers need `border-separate`.** Under the default
  `border-collapse: collapse`, the browser takes ownership of cell borders
  and a sticky header loses its underline as rows slide beneath it. Use
  `border-separate border-spacing-0` on the table and put borders on the
  cells (`border-b` on `TableHead` / `TableCell`), not on the `<tr>`.
- **`overflow-hidden` on a wrapper breaks sticky inside it.** It makes that
  wrapper the scrollport, so a sticky header pins to the wrapper instead of
  the real scroll region. If you need rounded corners on a table panel,
  round the header cells (`first:rounded-tl-lg last:rounded-tr-lg`) instead
  of clipping the wrapper.
- **shadcn's `Table` wraps itself in a div you can't reach via `className`**
  — `className` lands on the `<table>`. This repo's copy adds a
  `containerClassName` prop for the wrapper; that's the one to use for
  `h-full` / `min-h-0 flex-1` scroll regions.
- **shadcn's `Item` can't do single-line truncation.** `Item` is `flex-wrap`
  and `ItemTitle` sets `flex` and `line-clamp-1` on the same element, so a
  long title wraps to a second line instead of clipping. Sidebar rows in
  `PanelList.tsx` use a ghost `Button` instead — don't "fix" them onto `Item`.
- **A row action can't be nested inside a row button.** `<button>` inside
  `<button>` is invalid HTML and the inner one gets dropped. Follow shadcn's
  sidebar-menu shape: the action is a *sibling*, absolutely positioned over
  the row's right edge, with matching `pr-*` on the row so a truncated label
  never slides underneath it.
- **`@shadcn/sidebar` is radix-based.** This project is base-ui (`base-nova`
  style); installing it would pull a second primitive library in. Rebuild its
  menu shape from `Button` rather than adding it.
- **base-ui `ToggleGroup` clears its value** when you click the already-
  selected item. Guard with `if (next)` in `onValueChange` or the control
  ends up with nothing selected.
- **Recharts gradient ids collide** when two charts mount at once. Derive
  them from `useId()`, as `MetricChart` does.

## How to run this check

1. Get the diff: `git diff` (or the specific files just written/edited).
2. Walk each changed `.tsx`/`.css` file's `className`s and inline styles
   against sections 0–7 above.
3. Report findings as a short list: file, line/snippet, which rule it
   violates, and the fix (usually "use X instead"). Don't flag intentional,
   already-established exceptions (e.g. `text-[#5a9184]` on the Welcome
   page's "Keeping Tabs" slogan is a deliberate one-off brand accent, not
   drift — use judgment, the goal is catching *accidental* inconsistency,
   not enforcing zero arbitrary values ever).
4. If nothing violates the checklist, say so plainly — don't pad the
   report with restating what's already correct.
