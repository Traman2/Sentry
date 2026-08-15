# Contributing to System-Expert

Thanks for taking a look at System-Expert. This doc covers how the repo is laid out,
how to get a dev environment running, and what's expected of a pull request.

## Project layout

System-Expert is a Cargo workspace plus one external Python component:

```
crates/
  system-expert-core/       system data collection (library, no UI)
  system-expert-tauri-ui/    Tauri + React desktop app — the only binary
    src/              React frontend
    src-tauri/         Rust backend + in-process MCP server
agent/                external LangGraph agent (Python, not in the Cargo workspace)
```

See the [README](README.md) for the full architecture writeup before diving
into a change that crosses crate boundaries.

## Getting set up

You'll need:

- Rust (stable) with `rustfmt` and `clippy`
- Node.js 20+
- Python 3.11+ (only if you're touching the agent)

```bash
# Rust workspace
cargo check --workspace

# Desktop app frontend
cd crates/system-expert-tauri-ui
npm install
npm run tauri dev

# Python agent (optional, only needed for chat features)
cd agent
pip install -r requirements.txt
cp .env.example .env   # add your GROQ_API_KEY
python app.py --serve
```

## Before opening a pull request

CI (`.github/workflows/rust-ci.yml`) runs the following on every PR — run
them locally first so you're not waiting on a red build:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets
cargo test --workspace
```

For frontend changes:

```bash
cd crates/system-expert-tauri-ui
npm run build   # tsc + vite build
```

A few conventions this codebase follows — please match them rather than
introducing a new style in a single file:

- No speculative abstractions or config flags for hypothetical future needs —
  see the root `CLAUDE.md` / project norms if you're using an AI assistant
  to help write the change.
- Comments explain *why*, not *what* — skip comments that just restate the
  code.
- Keep `system-expert-core` free of UI and network concerns; it's a plain data
  library. New capabilities exposed to the agent go through the MCP layer
  in `crates/system-expert-tauri-ui/src-tauri/src/mcp/`.

## Filing issues

Use the issue templates under **New Issue** — they're split by component
(`system-expert-core`, `system-expert-tauri-ui`, MCP server, Python agent) so bug reports
land with the right context up front. Search existing issues first to avoid
duplicates.

## Pull requests

1. Fork the repo and create a branch off `master`.
2. Keep the PR focused — one logical change per PR is easier to review and
   easier to revert if something goes wrong.
3. Update the [CHANGELOG](CHANGELOG.md) under an "Unreleased" heading if your
   change is user-facing.
4. Make sure CI is green.
5. Open the PR against `master` with a description of *why* the change is
   needed, not just what it does — link an issue if one exists.

## Code of conduct

Be respectful and constructive in issues, PRs, and reviews. Disagreement
about approach is normal; keep it about the code.

## License

By contributing, you agree that your contributions will be licensed under
the [MIT License](LICENSE).
