"""The MCP connection and direct tool access.

`langchain-mcp-adapters` turns whatever the Rust side publishes into LangChain tools, so a
tool added in `crates/sentry-tauri-ui/src-tauri/src/mcp/tools.rs` reaches the agent with no
change here.
"""

from __future__ import annotations

import json
from typing import Any

from langchain_mcp_adapters.client import MultiServerMCPClient


class SentryTools:
    """Loads the desktop app's MCP tools and calls them directly when needed."""

    def __init__(self, url: str) -> None:
        self.url = url
        self._client = MultiServerMCPClient(
            {"sentry": {"transport": "http", "url": url}}
        )
        self._tools: list[Any] = []

    @property
    def tools(self) -> list[Any]:
        return self._tools

    async def load(self) -> list[Any]:
        self._tools = await self._client.get_tools()
        return self._tools

    async def call(self, name: str, **kwargs: Any) -> Any:
        """Invoke one tool directly, bypassing the model.

        The chat bridge uses this for its bookkeeping reads and for posting replies, so those
        stay deterministic instead of depending on the model choosing to call them.
        """
        tool = next((t for t in self._tools if t.name == name), None)
        if tool is None:
            raise LookupError(f"no such tool: {name}")
        return decode(await tool.ainvoke(kwargs))


def decode(raw: Any) -> Any:
    """Unwrap an MCP tool response into the payload the server actually sent.

    Responses arrive as content blocks (`[{"type": "text", "text": ...}]`), not the tool's
    return value. Tools that fail at the tool level — a refused kill, a missing id — send
    prose rather than JSON, so a decode failure means "read this as a message", not "this
    broke".
    """
    if isinstance(raw, list):
        text = "\n".join(
            block.get("text", "")
            for block in raw
            if isinstance(block, dict) and block.get("type") == "text"
        )
    elif isinstance(raw, str):
        text = raw
    else:
        return raw

    try:
        return json.loads(text)
    except json.JSONDecodeError:
        return text
