//! Guardrails on what `kill_process` is allowed to terminate.

/// Windows processes that terminating would take the machine down with them. Killing any of
/// these requires `force: true`, which exists as an escape hatch for a caller that genuinely
/// knows better rather than as a thing to reach for.
const CRITICAL_PROCESS_NAMES: &[&str] = &[
    "system",
    "registry",
    "smss.exe",
    "csrss.exe",
    "wininit.exe",
    "winlogon.exe",
    "services.exe",
    "lsass.exe",
];

/// Checks a kill request against the guardrails, returning `Some(reason)` if it should be
/// refused.
///
/// These live here rather than in `sentry-core` on purpose: the core library stays a neutral
/// mechanism, and policy about what an *agent* is allowed to terminate belongs at the edge
/// that exposes it. `force` overrides everything except pid 0, which is never a real
/// killable process on any supported platform.
pub fn kill_guard(pid: u32, name: Option<&str>, force: bool) -> Option<String> {
    if pid == 0 {
        return Some("pid 0 is not a real process".to_string());
    }

    if force {
        return None;
    }

    if pid == std::process::id() {
        return Some(format!(
            "pid {pid} is Sentry itself — killing it would take this MCP server down with it; \
             pass force: true if that is genuinely what you want"
        ));
    }

    #[cfg(windows)]
    if pid == 4 {
        return Some(
            "pid 4 is the Windows System process — terminating it bugchecks the machine; \
             pass force: true to override"
                .to_string(),
        );
    }

    if let Some(name) = name {
        let lowered = name.to_lowercase();
        if CRITICAL_PROCESS_NAMES.contains(&lowered.as_str()) {
            return Some(format!(
                "{name} is a critical OS process — terminating it will destabilise or reboot the \
                 machine; pass force: true to override"
            ));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kill_guard_always_refuses_pid_zero_even_with_force() {
        assert!(kill_guard(0, None, true).is_some());
    }

    #[test]
    fn kill_guard_refuses_sentry_itself_unless_forced() {
        let me = std::process::id();
        assert!(kill_guard(me, None, false).is_some());
        assert!(kill_guard(me, None, true).is_none());
    }

    #[test]
    fn kill_guard_refuses_critical_names_case_insensitively() {
        assert!(kill_guard(1234, Some("LSASS.EXE"), false).is_some());
        assert!(kill_guard(1234, Some("lsass.exe"), true).is_none());
        assert!(kill_guard(1234, Some("notepad.exe"), false).is_none());
    }
}
