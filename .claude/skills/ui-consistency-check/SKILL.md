---
name: ui-consistency-check
description: Review new or changed frontend code in crates/sentry-tauri-ui against this app's established design tokens, spacing, and component-reuse conventions, and report deviations. Use after writing or editing any .tsx/.css in crates/sentry-tauri-ui, or whenever the user asks whether the UI is "on brand", "consistent", or "flush with the rest of the app".
---

# UI Consistency Check

This skill exists so reviewing UI work doesn't require re-reading the whole
frontend to remember what "normal" looks like — the rules below **are**
that context, extracted once from `index.css` and the existing components.
Check changed code against this list directly; only open other files when
you need to confirm how a specific existing pattern is actually used.

Scope: `crates/sentry-tauri-ui/src/`.

## 1. Color tokens

Source of truth: `crates/sentry-tauri-ui/src/index.css`. Four brand colors,
nothing else should appear as a raw hex/rgb color in component code:

| Token | Value | Use for |
| --- | --- | --- |
| `canvas` | `#ffffff` | surfaces/backgrounds (`bg-canvas`) |
| `mint` | `#dbe8c8` | rarely used directly; backs `--muted` |
| `teal` | `#93c3b8` | borders, dividers, hover fills, secondary/accent (`border-teal`, `bg-teal/10`–`/40`, `text-teal`) |
| `navy` | `#26264f` | text, icons, primary actions (`text-navy`, `bg-navy` for primary buttons) |
| `danger` | `#dc2626` | destructive-only (kill/delete actions, high CPU/mem warnings) |

Semantic Tailwind classes (`bg-primary`, `text-muted-foreground`,
`bg-accent`, `border-border`, etc.) already resolve to these four via the
`:root` block in `index.css` — prefer them for shadcn/base-ui primitives
(`components/ui/*`), but plain feature code overwhelmingly uses the brand
names directly (`text-navy`, `border-teal/50`) rather than the semantic
aliases. Match whichever convention the file you're editing already uses;
don't mix both in one component.

**Flag**: any `#RRGGBB`, `rgb(...)`, or named CSS color in a `className`
or inline `style` that isn't one of the four above (or a `rgba()` build
directly off `--color-navy`'s `38,38,79`, as used for `bg-dot-pattern` and
the scrollbar). One-off arbitrary colors are the single most common way
new UI drifts off-brand.

**Health/status color** (CPU/memory ring, load indicators): don't invent a
new threshold scheme — reuse `healthColor()` from `@/lib/health.ts` (>=50%
→ danger, >=15% → amber `#c9a227`, else green `#3fa66b`) and the `Gauge`
component (`@/components/Gauge.tsx`) that renders it as a ring. If you're
about to write a `<circle>`/`<svg>` gauge or a percent-to-color function
inline, stop — import these instead.

## 2. Typography scale

| Role | Classes | Example |
| --- | --- | --- |
| Table/section header label | `text-[10px] font-semibold uppercase tracking-wide text-navy/60` | column headers, stat card labels |
| Primary body text | `text-xs font-medium text-navy` (or `text-sm` for modal-scale content) | row titles, values |
| Secondary/meta text | `text-[11px] text-muted-foreground` | subtitles, "PID 1234", captions |
| Modal/page heading | `text-lg`–`text-xl font-semibold` via `DialogTitle`, or `text-4xl font-bold` for tab-page hero headings (`Welcome.tsx`) | modal titles, page titles |

**Flag**: ad-hoc font sizes outside `text-[10px]`, `text-[11px]`, `text-xs`,
`text-sm`, `text-lg`, `text-xl`, the heading sizes, or arbitrary values like
`text-[13px]` invented for one component.

## 3. Spacing & sizing

- Tailwind's default 4px scale throughout (`gap-1.5`, `px-3`, `py-1.5`,
  `p-6`, etc.) — no arbitrary pixel spacing (`p-[13px]`) unless matching a
  hard external constraint (icon size, scrollbar width compensation).
- Card/box padding: `p-3` (compact, sidebar cards) to `p-6` (modal
  sections). `p-4` for mid-weight boxes (status panels).
- Fixed component widths use the same 4px-multiple arbitrary scale as
  existing modals: `w-96` (small modal), `w-160` (large modal, = 640px),
  `w-125`/`h-125` (500px, Welcome page box). Reuse these rather than
  inventing a new width for a similarly-scoped component.
- Rounded corners: `rounded-md` (buttons, small controls), `rounded-lg`
  (cards, table boxes, tab-page root), `rounded-xl` (modal card root).

## 4. Borders, surfaces, elevation

- Card/box border: `border border-teal/50` (subtle) or `border border-teal`
  (stronger, e.g. tab-page root, modal root).
- Card fill: `bg-canvas` (default) or `bg-teal/5`–`/10` (subtly tinted,
  e.g. status panels, hovered rows).
- Row hover: `hover:bg-teal/10` (table rows) or `hover:bg-teal/25` (buttons,
  menu items, toolbar controls).
- Shadow: `shadow-sm` (tab-page root, side panel), `shadow-lg`/`shadow-xl`
  (modals — the more "on top of everything" a surface is, the heavier the
  shadow).
- Dividers inside a box: `border-b border-teal/30` between rows,
  `border-b border-teal/50` for a header/footer separator from body content.

**Flag**: a new bordered box that doesn't use `border-teal` at some
opacity, or a hover state that isn't `teal`-based.

## 5. Component reuse checklist

Before writing new markup, check whether it already exists:

- Button → `@/components/ui/button.tsx` (`Button`, variants:
  `default`/`outline`/`secondary`/`ghost`/`destructive`/`link`, sizes
  `xs`/`sm`/`default`/`lg`/`icon*`). Never a raw styled `<button>` for
  anything that reads as a primary/secondary action — raw `<button>` is
  only for lightweight inline controls (table sort toggle, close icon)
  that don't fit the Button's chrome.
- Dropdown/menu → `@/components/ui/dropdown-menu.tsx` /
  `menubar.tsx`.
- Dialog/modal chrome → `@/components/ui/dialog.tsx`
  (`DialogTitle`, `DialogClose`, etc.), inside a component in `modals/`
  opened via `useModalStore().openModal(...)`. Never build a second modal
  mechanism.
- CPU/percent ring → `@/components/Gauge.tsx`.
- Byte/rate/percent formatting → `@/tab-pages/ResourceMonitor/format.ts`
  (`formatBytes`, `formatRate`, `formatPercent`). Never re-implement
  "divide by 1024 until small" inline.
- Process/app aggregation → `groupProcessesByApp`, `findAppRowByPid` in
  `@/tab-pages/ResourceMonitor/groupProcesses.ts`.
- `cn()` from `@/lib/utils` for any conditional/merged className — never
  string-template classNames with manual ternaries when `cn` does it
  cleanly (small, but it's the established idiom in every `components/ui/*`
  file).

**Flag**: an inline SVG gauge, a hand-rolled byte formatter, a second
zustand-store-less local implementation of something a store already
does (esp. tabs/modal/panel open-close state), or a raw `@base-ui/react`
import inside a feature file instead of the wrapped `components/ui/`
version.

## 6. Layout patterns worth matching exactly

- **Scrollable region inside a fixed-height container** (modals, panels):
  pinned header/footer (`shrink-0`) + `flex-1 overflow-y-auto` middle —
  see `component-scaffold`'s modal skeleton. Don't use `h-full` +
  percentage heights nested inside flex — it silently breaks scrolling
  (this exact bug happened once already in `ViewMoreModal.tsx`'s history).
- **Tab page root**: always
  `flex h-full w-full flex-col overflow-hidden rounded-lg border border-teal bg-canvas shadow-sm`.
- **Never import a tab page's own config as a value from within that tab
  page** (circular import — see `component-scaffold` for the full
  explanation and the workaround).

## How to run this check

1. Get the diff: `git diff` (or the specific files just written/edited).
2. Walk each changed `.tsx`/`.css` file's `className`s and inline styles
   against sections 1–6 above.
3. Report findings as a short list: file, line/snippet, which rule it
   violates, and the fix (usually "use X instead"). Don't flag intentional,
   already-established exceptions (e.g. `text-[#5a9184]` on the Welcome
   page's "Keeping Tabs" slogan is a deliberate one-off brand accent, not
   drift — use judgment, the goal is catching *accidental* inconsistency,
   not enforcing zero arbitrary values ever).
4. If nothing violates the checklist, say so plainly — don't pad the
   report with restating what's already correct.
