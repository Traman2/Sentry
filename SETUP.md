# Sentry Workspace Setup

Steps to scaffold the `sentry` Cargo workspace with three crates plus a
standalone Python agent directory. Run these from `C:\TejasProjects\Sentry`.

## 1. Create `sentry-core` (lib crate, no Tauri/MCP deps)

```
cargo new --lib crates/sentry-core
```

Add to `crates/sentry-core/Cargo.toml`:

```toml
[dependencies]
sysinfo = "0.32"
```

Leave `src/lib.rs` as a stub (default `cargo new` contents, or an empty
`pub fn placeholder() {}`) — no real monitoring logic yet.

## 2. Create `sentry-mcp` (bin crate, MCP server)

```
cargo new --bin crates/sentry-mcp
```

Add to `crates/sentry-mcp/Cargo.toml`:

```toml
[dependencies]
sentry-core = { path = "../sentry-core" }
rmcp = "0.1"        # verify latest version on crates.io before pinning
tokio = { version = "1", features = ["full"] }
```

Leave `src/main.rs` as a stub `fn main() {}` for now. This crate is meant to
be publishable to crates.io standalone eventually, so keep its dependency
list minimal and don't let it depend on the Tauri crate.

## 3. Scaffold `sentry-tauri` via `cargo create-tauri-app`

Do **not** use `cargo new` for this one.

First install the scaffolding tool if it isn't already available:

```
cargo install create-tauri-app
```

Then, from `crates/`:

```
cd crates
cargo create-tauri-app sentry-tauri --template react-ts --manager npm
```

The `--template react-ts` flag scaffolds the **React + TypeScript**
frontend non-interactively (skip the prompts). This project is meant to
render a process table, so React is required here — later this becomes the
base for a `react-window` (or `@tanstack/react-virtual`) virtualized table.

If your installed version of `create-tauri-app` doesn't support these
flags, run `cargo create-tauri-app sentry-tauri` without them and answer
the prompts by choosing **React** as the frontend framework and
**TypeScript** as the flavor.

This generates `crates/sentry-tauri/src-tauri/` containing the actual Rust
crate (`Cargo.toml`, `src/main.rs`, etc.) alongside the React frontend
project files (`crates/sentry-tauri/src/`, `package.json`, etc.).

After scaffolding, install `react-window` in the frontend:

```
cd sentry-tauri
npm install react-window
npm install --save-dev @types/react-window
```

`react-window` is a frontend/npm dependency only — it has no bearing on the
Rust `Cargo.toml` or the workspace build.

After scaffolding, edit `crates/sentry-tauri/src-tauri/Cargo.toml` to add:

```toml
[dependencies]
sentry-core = { path = "../../sentry-core" }
```

Do **not** add MCP or agent-related dependencies here — this crate only
renders a process table using `sentry-core`.

## 4. Root workspace `Cargo.toml`

Create `C:\TejasProjects\Sentry\Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "crates/sentry-core",
    "crates/sentry-mcp",
    "crates/sentry-tauri/src-tauri",
]
```

## 5. Verify the build

```
cargo build
```

This should succeed with stub implementations across all three crates
before any real logic is filled in.

## 6. Create `agent/` directory (Python MCP client, outside the workspace)

This is **not** a Rust crate and must NOT be added to the workspace
`members` list.

```
mkdir agent
```

Add `agent/pyproject.toml`:

```toml
[project]
name = "sentry-agent"
version = "0.1.0"
description = "Python MCP client for sentry-mcp"
requires-python = ">=3.11"
dependencies = [
    "mcp",
]

[build-system]
requires = ["hatchling"]
build-backend = "hatchling.build"
```

Leave implementation files (e.g. `agent/main.py` or `agent/src/...`) as
stubs for now — no client logic yet.

Create and activate a virtual environment inside `agent/`:

```
cd agent
python -m venv .venv
```

Activate it (PowerShell):

```
C:\TejasProjects\Sentry\agent\.venv\Scripts\activate
```

Then install the project in editable mode:

```
pip install -e .
```

## Notes / order of operations

- Do steps 1–2 first (`cargo new` for sentry-core and sentry-mcp), then
  step 3 (`cargo create-tauri-app`), then step 4 (root workspace
  `Cargo.toml`), then verify with `cargo build` (step 5).
- `agent/` (step 6) is independent and can be done at any point since it's
  outside the Cargo workspace entirely.
- No monitoring, MCP tool, or UI logic should be implemented yet — this is
  purely scaffolding to get `cargo build` green across the workspace.