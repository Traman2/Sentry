//! Launching and supervising the Python agent.
//!
//! The desktop app owns the agent's lifetime: it starts it once the MCP server is listening
//! and kills it on exit. That means a user never has to run anything by hand for the chat
//! panel to work, and — just as importantly — never ends up with an orphaned agent still
//! polling a database after the app it belonged to is gone.
//!
//! This is a development-oriented path. It runs `agent/app.py` out of the source tree with a
//! Python interpreter found on this machine; a packaged build has no `agent/` directory and
//! reports [`AgentStatus::detail`] saying so rather than failing loudly. Shipping the agent
//! to end users would mean bundling an interpreter as a Tauri sidecar, which is a separate
//! piece of work.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;

use serde::Serialize;

/// Overrides where the agent source is looked for. Useful when the app runs from somewhere
/// that isn't inside the repo.
const AGENT_DIR_ENV: &str = "SENTRY_AGENT_DIR";

#[derive(Debug, Clone, Serialize)]
pub struct AgentStatus {
    pub running: bool,
    pub pid: Option<u32>,
    /// The model key the agent was last asked to use.
    pub model: String,
    /// Human-readable state, shown in the UI when something isn't right.
    pub detail: String,
}

/// A handle to the agent child process.
pub struct AgentProcess {
    child: Option<Child>,
    model: String,
    detail: String,
}

impl AgentProcess {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            child: None,
            model: model.into(),
            detail: "not started".to_string(),
        }
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// Records the model the agent should use.
    ///
    /// Does **not** restart anything: the running agent reads this back over MCP through
    /// `get_agent_config` on its next poll and swaps models in place, which keeps every
    /// conversation's history. Restarting to change models would throw that away.
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = model.into();
    }

    /// Whether the child is still alive, reaping it if it has exited.
    ///
    /// Takes `&mut self` because finding out requires `try_wait`, which also reaps — without
    /// that, a crashed agent lingers as a zombie and still looks like it's running.
    pub fn is_running(&mut self) -> bool {
        let Some(child) = self.child.as_mut() else {
            return false;
        };
        match child.try_wait() {
            Ok(Some(status)) => {
                self.detail = format!("agent exited ({status})");
                self.child = None;
                false
            }
            Ok(None) => true,
            Err(e) => {
                self.detail = format!("could not check agent: {e}");
                self.child = None;
                false
            }
        }
    }

    pub fn status(&mut self) -> AgentStatus {
        let running = self.is_running();
        AgentStatus {
            running,
            pid: running.then(|| self.child.as_ref().map(|c| c.id())).flatten(),
            model: self.model.clone(),
            detail: if running {
                "running".to_string()
            } else {
                self.detail.clone()
            },
        }
    }

    /// Starts the agent, replacing any process already running.
    pub fn start(&mut self) -> Result<AgentStatus, String> {
        self.stop();

        let dir = agent_dir().ok_or_else(|| {
            format!(
                "could not find the agent source (looked for agent/app.py near the executable; \
                 set {AGENT_DIR_ENV} to override)"
            )
        })?;
        let python = python_for(&dir).ok_or_else(|| {
            format!("no Python interpreter found for {}", dir.display())
        })?;

        let child = Command::new(&python)
            .arg("app.py")
            .arg("--serve")
            .arg("--model")
            .arg(&self.model)
            // cwd is the agent directory so `load_dotenv()` picks up agent/.env, which is
            // where the API key lives.
            .current_dir(&dir)
            // Unbuffered, or the agent's progress output sits in a block buffer forever now
            // that its stdout is a pipe rather than a terminal.
            .env("PYTHONUNBUFFERED", "1")
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to start {}: {e}", python.display()))?;

        let pid = child.id();
        self.child = Some(child);
        self.detail = "running".to_string();
        println!(
            "sentry-agent: started pid {pid} ({} --serve --model {})",
            python.display(),
            self.model
        );
        Ok(self.status())
    }

    /// Kills the agent if it's running. Safe to call when it isn't.
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            // Reap, so we don't leave a zombie behind.
            let _ = child.wait();
            println!("sentry-agent: stopped");
        }
        self.detail = "stopped".to_string();
    }
}

impl Drop for AgentProcess {
    /// Backstop so the agent cannot outlive the app even on an unusual shutdown path.
    fn drop(&mut self) {
        self.stop();
    }
}

/// Tauri-managed wrapper.
pub struct AgentState(pub Mutex<AgentProcess>);

impl AgentState {
    pub fn new(model: &str) -> Self {
        Self(Mutex::new(AgentProcess::new(model)))
    }

    pub fn lock(&self) -> std::sync::MutexGuard<'_, AgentProcess> {
        self.0.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Locates the `agent/` directory by walking up from the executable.
///
/// In development the binary sits at `target/debug/`, so the repo root is a few levels up;
/// the walk avoids hardcoding how many. Returns `None` in a packaged build, where there is
/// no agent source at all.
fn agent_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os(AGENT_DIR_ENV) {
        let dir = PathBuf::from(dir);
        return dir.join("app.py").is_file().then_some(dir);
    }

    let exe = std::env::current_exe().ok()?;
    let mut cursor: &Path = exe.parent()?;
    loop {
        let candidate = cursor.join("agent");
        if candidate.join("app.py").is_file() {
            return Some(candidate);
        }
        cursor = cursor.parent()?;
    }
}

/// Picks an interpreter: the agent's virtualenv if present, otherwise whatever is on PATH.
///
/// The venv is strongly preferred — that's where the dependencies actually are, so falling
/// through to a bare `python` usually only produces an ImportError, but it is still a better
/// failure than refusing to try.
fn python_for(dir: &Path) -> Option<PathBuf> {
    let venv = if cfg!(windows) {
        dir.join(".venv").join("Scripts").join("python.exe")
    } else {
        dir.join(".venv").join("bin").join("python")
    };
    if venv.is_file() {
        return Some(venv);
    }
    Some(PathBuf::from(if cfg!(windows) { "python" } else { "python3" }))
}
