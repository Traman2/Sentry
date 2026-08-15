"""The LangGraph agent and its hot-swappable model."""

from __future__ import annotations

import asyncio
from collections.abc import Awaitable, Callable
from contextlib import AsyncExitStack
from typing import Any

from langchain.agents import create_agent
from langgraph.checkpoint.sqlite.aio import AsyncSqliteSaver

from . import models
from .config import CHECKPOINT_DB, SYSTEM_PROMPT
from .models import ModelSpec
from .tools import SystemExpertTools


class SystemExpertAgent:
    """A LangGraph agent wired to the desktop app's MCP tools.

    The graph is built lazily so `--list-tools` and `--list-models` work without an API key,
    and rebuilt on [`SystemExpertAgent.set_model`] — the checkpointer outlives the rebuild, so a
    model swap keeps every conversation's history rather than resetting it.
    """

    def __init__(self, url: str, model: str | None = None) -> None:
        self.tools = SystemExpertTools(url)
        self._spec = models.resolve(model)
        self._graph = None
        # Threads are keyed by chat space id, so each conversation in the app's UI carries
        # its own history instead of us replaying the transcript on every turn. Backed by
        # SQLite rather than memory so a restarted agent (a crash, `--serve` relaunching)
        # picks a conversation back up instead of the model losing its grounding while the
        # desktop app's own transcript — a separate store — still shows the earlier turns.
        # Opened lazily because `AsyncSqliteSaver` needs a running event loop; closed via
        # `aclose()`, which callers should invoke once when the process is shutting down.
        # Guarded by a lock because `--serve` answers every chat space concurrently (see
        # `chat_bridge.reconcile`) — without it, two spaces both hitting a cold start would
        # each pass the `is None` check before either finished `await`ing, opening two
        # connections to the same file and (on the desktop app's Windows default, no WAL)
        # having the loser fail with "database is locked".
        self._checkpointer: AsyncSqliteSaver | None = None
        self._checkpointer_lock = asyncio.Lock()
        self._exit_stack = AsyncExitStack()
        self._graph_lock = asyncio.Lock()

    async def aclose(self) -> None:
        """Closes the checkpointer's database connection. Idempotent."""
        await self._exit_stack.aclose()

    async def _get_checkpointer(self) -> AsyncSqliteSaver:
        async with self._checkpointer_lock:
            if self._checkpointer is None:
                saver = await self._exit_stack.enter_async_context(
                    AsyncSqliteSaver.from_conn_string(str(CHECKPOINT_DB))
                )
                await saver.setup()
                self._checkpointer = saver
            return self._checkpointer

    @property
    def url(self) -> str:
        return self.tools.url

    @property
    def spec(self) -> ModelSpec:
        return self._spec

    async def load_tools(self) -> list[Any]:
        return await self.tools.load()

    def set_model(self, name: str, *, strict: bool = False) -> ModelSpec:
        """Switch models. Raises `UnknownModelError` for an unrecognised name.

        Only drops the compiled graph — conversation state lives in the checkpointer, so the
        next turn continues where the previous model left off.

        `strict` is for names coming from the desktop app; see `models.resolve`.
        """
        spec = models.resolve(name, strict=strict)
        if spec.model_id != self._spec.model_id:
            self._spec = spec
            self._graph = None
        return spec

    async def graph(self):
        async with self._graph_lock:
            if self._graph is None:
                if not self.tools.tools:
                    await self.load_tools()
                checkpointer = await self._get_checkpointer()
                self._graph = create_agent(
                    models.build_model(self._spec),
                    self.tools.tools,
                    system_prompt=SYSTEM_PROMPT,
                    checkpointer=checkpointer,
                )
            return self._graph

    async def ask(
        self,
        question: str,
        thread_id: str = "cli",
        on_step: Callable[[str], Awaitable[None]] | None = None,
    ) -> str:
        """Run one turn, reporting each step through `on_step` as it happens.

        Streamed rather than invoked so the caller can show progress: a turn that reads
        several tools takes long enough that an unexplained wait is the wrong thing to put in
        front of someone. The final answer is the last assistant text the graph produces.
        """
        graph = await self.graph()
        final = ""

        async for update in graph.astream(
            {"messages": [{"role": "user", "content": question}]},
            config={"configurable": {"thread_id": thread_id}},
            stream_mode="updates",
        ):
            # `updates` is {node_name: {"messages": [...]}}. Inspecting the messages rather
            # than keying off node names keeps this working if the graph's internals are
            # renamed.
            for node_update in update.values():
                if not isinstance(node_update, dict):
                    continue
                for message in node_update.get("messages") or []:
                    if on_step:
                        for step in describe_steps(message):
                            await on_step(step)
                    if getattr(message, "type", None) == "ai":
                        text = message_text(message)
                        if text:
                            final = text

        return final


def message_text(message: Any) -> str:
    """The plain text of a message, skipping reasoning blocks."""
    content = getattr(message, "content", "")
    if isinstance(content, str):
        return content.strip()
    # Reasoning models return a list of blocks; keep the text ones.
    parts = [
        block.get("text", "")
        for block in content
        if isinstance(block, dict) and block.get("type") == "text"
    ]
    return "\n".join(p for p in parts if p).strip()


def describe_steps(message: Any) -> list[str]:
    """Turn one streamed message into the progress lines a user would want to read.

    Tool calls are the honest unit of progress here — they are what actually takes time and
    what the answer is grounded in. Everything is read defensively because message shapes
    vary by provider, and a missing attribute should cost a progress line, not the turn.
    """
    kind = getattr(message, "type", None)

    if kind == "ai":
        steps = []
        # gpt-oss and friends expose their reasoning separately from the reply.
        reasoning = (getattr(message, "additional_kwargs", None) or {}).get(
            "reasoning_content"
        )
        if isinstance(reasoning, str) and reasoning.strip():
            steps.append(_first_sentence(reasoning))
        for call in getattr(message, "tool_calls", None) or []:
            name = call.get("name") if isinstance(call, dict) else None
            if name:
                steps.append(f"Calling {name}")
        return steps

    if kind == "tool":
        name = getattr(message, "name", None) or "tool"
        return [f"Read {name}"]

    return []


def _first_sentence(text: str, limit: int = 140) -> str:
    """A single readable line from a block of reasoning."""
    line = " ".join(text.split())
    if len(line) <= limit:
        return line
    return line[:limit].rsplit(" ", 1)[0] + "…"
