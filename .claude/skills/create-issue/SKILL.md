---
name: create-issue
description: File a GitHub issue against this repo (System-Expert) using the component issue templates in .github/ISSUE_TEMPLATE/ — system-expert-core, system-expert-tauri-ui, system-expert-mcp, or python-agent. Use when the user wants to open/file/create a GitHub issue, report a bug, or request a feature for one of the four system components.
---

# Create Issue

Files a GitHub issue via `gh issue create`, matching the structure of one of
the four component templates under `.github/ISSUE_TEMPLATE/`:

| Template file | Component | Label |
| --- | --- | --- |
| `system-expert-core.yml` | Data collection library (`crates/system-expert-core`) | `system-expert-core` |
| `system-expert-tauri-ui.yml` | Desktop app, frontend + Tauri backend (`crates/system-expert-tauri-ui`) | `system-expert-tauri-ui` |
| `system-expert-mcp.yml` | In-process MCP server (`crates/system-expert-tauri-ui/src-tauri/src/mcp`) | `system-expert-mcp` |
| `python-agent.yml` | External LangGraph MCP agent (`agent/`) | `python-agent` |

## Steps

1. **Identify the component.** If the user didn't say which of the four
   components this is about, ask (use AskUserQuestion with the four options
   above) — don't guess from a vague description.

2. **Read the matching template** at
   `.github/ISSUE_TEMPLATE/<component>.yml` to see its exact fields (e.g.
   `system-expert-tauri-ui.yml` has an `Area` dropdown, `system-expert-core.yml` has an
   `OS / platform` field). Gather the info for each field from the
   conversation; ask the user for anything required (`validations.required:
   true`) that's still missing. Skip optional fields the user doesn't have
   info for.

3. **Build the issue body** as Markdown, using each field's `label` as a
   `##` heading followed by the user's answer, in the same order as the
   template. Skip empty optional fields entirely rather than leaving them
   blank.

4. **Title**: prefix with the template's `title` value (e.g. `[system-expert-core]
   `) followed by a short summary of the issue.

5. **Confirm with the user** before creating anything — show them the
   title, body, and label you're about to submit. Creating a GitHub issue
   is a visible, hard-to-fully-undo action (it can be closed but not
   unfiled), so always get explicit go-ahead first, even if they asked you
   to "just file it."

6. **Create it**:
   ```
   gh issue create --title "<title>" --body "<body>" --label "<component-label>"
   ```
   Run this from inside the repo (`C:\TejasProjects\Sentry`) so `gh` infers
   the repo (`Traman2/Sentry`) from the git remote — no need to pass
   `--repo`.

7. Report back the issue URL `gh issue create` prints on success.

## Notes

- If `gh` isn't authenticated, `gh issue create` will fail with an auth
  error — tell the user to run `gh auth login` themselves rather than
  attempting it for them.
- Don't invent labels beyond the one matching the component; if the repo
  doesn't have that label yet, `gh` will error — in that case ask the user
  whether to create the label (`gh label create`) or file without one.