"""Sentry's LangGraph agent.

Connects to the MCP server hosted inside the Sentry desktop app and reasons over the tools it
exposes. Models are served by Groq and swappable at runtime — see `models`.

Normally the desktop app launches this itself and manages its lifetime; running it by hand is
for development and for `--ask` one-offs.

Layout:
    config       constants and the system prompt
    models       the model registry, resolution, and construction
    tools        the MCP connection and direct tool calls (agent -> app)
    events       the WebSocket the app pushes over (app -> agent)
    agent        the LangGraph agent and its hot-swappable model
    chat_bridge  answering messages sent from the app's chat panel
    cli          argument parsing and entry point
"""

from .agent import SentryAgent
from .config import MCP_URL
from .models import MODELS, ModelSpec
from .tools import SentryTools

__all__ = ["MCP_URL", "MODELS", "ModelSpec", "SentryAgent", "SentryTools"]
