//! Turning what an MCP client tells us (`clientInfo`) and what the OS tells us (the process on
//! the other end of its socket) into one stable identity to group tool calls by.

/// What's known about the caller of one MCP tool call, gathered from two independent sources:
/// the MCP protocol handshake and OS-level process resolution. Either half can be missing —
/// `protocol_name` if a client's session hasn't completed `initialize` yet, `pid`/
/// `process_name` if the loopback port→pid lookup failed — and [`identify`] degrades
/// gracefully either way rather than dropping the call.
#[derive(Debug, Clone, Default)]
pub struct ClientIdentity {
    pub protocol_name: Option<String>,
    pub protocol_version: Option<String>,
    pub pid: Option<u32>,
    pub process_name: Option<String>,
    /// The resolved process's full command line, used only for [`classify`] — it's what
    /// tells the bundled Python agent apart from an unrelated `python.exe` some other MCP
    /// client happens to run as. Not stored in `mcp_clients`; `process_name` is.
    pub process_command: Option<String>,
}

/// A `clientInfo.name` value that doesn't actually identify anything — the generic default
/// some SDKs send in place of the embedding app naming itself. Treated as "no protocol name"
/// so these fall back to the process name instead of quietly merging distinct apps that all
/// happened to use their SDK's default under one meaningless shared key.
///
/// `"rmcp"` earned its spot here empirically: Claude Code's own MCP client — built on the
/// same `rmcp` crate this server uses — sends `clientInfo.name = "rmcp"` rather than
/// identifying itself as `"claude-code"`, confirmed by inspecting a live `mcp_clients` row
/// during development. Without this, every `rmcp`-based client would be keyed and displayed
/// as the literal string "rmcp" instead of falling back to its (far more informative)
/// process name.
fn is_generic_protocol_name(name: &str) -> bool {
    matches!(
        name.trim().to_ascii_lowercase().as_str(),
        "" | "mcp" | "mcp-client" | "mcp_client" | "python-sdk" | "python sdk" | "rmcp"
    )
}

/// The identity fields written to `mcp_clients`: `(key, display_name, kind)`.
///
/// `key` is what dedups repeated connections from the same logical client into one row —
/// prefer the client's own self-reported name (stable across reconnects, unlike a pid), fall
/// back to the process name only when the client didn't meaningfully self-identify.
pub fn identify(identity: &ClientIdentity) -> (String, String, String) {
    let protocol_name = identity
        .protocol_name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty() && !is_generic_protocol_name(n));

    let fallback = identity
        .process_name
        .as_deref()
        .map(str::trim)
        .filter(|n| !n.is_empty());

    let identifying_name = protocol_name.or(fallback);

    let key = identifying_name.unwrap_or("unknown").to_string();
    let display_name = identifying_name.unwrap_or("Unknown client").to_string();
    let kind = classify(
        protocol_name,
        identity.process_name.as_deref(),
        identity.process_command.as_deref(),
    );

    (key, display_name, kind)
}

/// Best-effort badge classification. Matched case-insensitively against every signal at once
/// so a client that only self-identifies weakly (e.g. a generic process name) can still be
/// caught by a distinctive command line, and vice versa. No match is `"unknown"` — this never
/// prevents a client from being recorded, it only affects its badge.
fn classify(
    protocol_name: Option<&str>,
    process_name: Option<&str>,
    process_command: Option<&str>,
) -> String {
    let haystack = format!(
        "{} {} {}",
        protocol_name.unwrap_or_default(),
        process_name.unwrap_or_default(),
        process_command.unwrap_or_default(),
    )
    .to_ascii_lowercase();

    const RULES: &[(&str, &str)] = &[
        ("claude-code", "claude-code"),
        ("claude code", "claude-code"),
        ("claude-desktop", "claude-desktop"),
        ("claude.exe", "claude-desktop"),
        ("chatgpt", "chatgpt"),
        ("openai", "chatgpt"),
        ("grok", "grok"),
        ("xai", "grok"),
        // The bundled agent's actual invocation (`crates/system-expert-tauri-ui/src-tauri/src/agent.rs`
        // spawns `<python> app.py --serve --model <name>`), confirmed against a live process's
        // command line during development — not a bare "python" match, since an unrelated
        // Python-based MCP client should land as "unknown", not be mistaken for this one.
        ("app.py --serve", "python-agent"),
        ("system_expert_agent", "python-agent"),
        ("system-expert-agent", "python-agent"),
    ];

    RULES
        .iter()
        .find(|(needle, _)| haystack.contains(needle))
        .map(|(_, kind)| kind.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_protocol_name_over_process_name() {
        let identity = ClientIdentity {
            protocol_name: Some("claude-code".to_string()),
            process_name: Some("claude.exe".to_string()),
            ..Default::default()
        };
        let (key, display_name, kind) = identify(&identity);
        assert_eq!(key, "claude-code");
        assert_eq!(display_name, "claude-code");
        assert_eq!(kind, "claude-code");
    }

    #[test]
    fn falls_back_to_process_name_when_protocol_name_is_generic() {
        // The actual invocation from `agent.rs`, confirmed against a live process during
        // development: `<python> app.py --serve --model <name>` from the agent's own
        // directory — not `python -m system_expert_agent`, which never appears on the command line.
        let identity = ClientIdentity {
            protocol_name: Some("mcp".to_string()),
            process_name: Some("python.exe".to_string()),
            process_command: Some(
                r"C:\TejasProjects\Sentry\agent\.venv\Scripts\python.exe app.py --serve --model qwen"
                    .to_string(),
            ),
            ..Default::default()
        };
        let (key, display_name, kind) = identify(&identity);
        assert_eq!(key, "python.exe");
        assert_eq!(display_name, "python.exe");
        assert_eq!(kind, "python-agent");
    }

    #[test]
    fn rmcp_protocol_name_is_treated_as_generic() {
        // Claude Code's own MCP client sends `clientInfo.name = "rmcp"` — the underlying
        // crate's name, not "claude-code" — confirmed against a live `mcp_clients` row during
        // development. Its process command line (an npm-installed
        // `@anthropic-ai/claude-code/bin/claude.exe`) is what actually identifies it.
        let identity = ClientIdentity {
            protocol_name: Some("rmcp".to_string()),
            protocol_version: Some("3.1.2".to_string()),
            process_name: Some("claude.exe".to_string()),
            process_command: Some(
                r"C:\Users\tejas\AppData\Roaming\npm\node_modules\@anthropic-ai\claude-code\bin\claude.exe"
                    .to_string(),
            ),
            ..Default::default()
        };
        let (key, display_name, kind) = identify(&identity);
        assert_eq!(key, "claude.exe");
        assert_eq!(display_name, "claude.exe");
        assert_eq!(kind, "claude-code");
    }

    #[test]
    fn falls_back_to_process_name_when_protocol_name_missing() {
        let identity = ClientIdentity {
            process_name: Some("Grok.exe".to_string()),
            ..Default::default()
        };
        let (key, _, kind) = identify(&identity);
        assert_eq!(key, "Grok.exe");
        assert_eq!(kind, "grok");
    }

    #[test]
    fn unknown_when_nothing_identifies_the_client() {
        let identity = ClientIdentity::default();
        let (key, display_name, kind) = identify(&identity);
        assert_eq!(key, "unknown");
        assert_eq!(display_name, "Unknown client");
        assert_eq!(kind, "unknown");
    }

    #[test]
    fn unrecognised_process_is_unknown_kind_but_still_keyed() {
        let identity = ClientIdentity {
            protocol_name: Some("some-other-tool".to_string()),
            ..Default::default()
        };
        let (key, _, kind) = identify(&identity);
        assert_eq!(key, "some-other-tool");
        assert_eq!(kind, "unknown");
    }
}
