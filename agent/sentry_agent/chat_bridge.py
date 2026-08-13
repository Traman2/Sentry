"""Answering messages the user sends from the desktop app's chat panel.

Event-driven: the app pushes over the WebSocket in `events` when a user sends a message or
changes the model, and we react. Tool calls still go over MCP — the socket carries only
notifications, never data.

One reconciliation sweep runs on connect, because messages sent while we were down produce
no event we could have received. After that, everything arrives as it happens.

The model is the app's to choose, not ours: the composer's picker is the single control, and
the app announces the current value on connect and on every change. `--model` only sets what
we start on, which matters for `--ask` but is superseded here.
"""

from __future__ import annotations

import asyncio
import sys
import traceback

import websockets

from .agent import SentryAgent
from .config import MAX_RECONNECT_ATTEMPTS, RECONNECT_SECONDS
from .events import stream
from .models import UnknownModelError

__all__ = ["describe", "pending_question", "report_error", "serve"]


def describe(exc: BaseException) -> str:
    """Flatten an exception to something a person can act on.

    The MCP client runs its transport in a `TaskGroup`, so a plain connection refusal
    surfaces as `unhandled errors in a TaskGroup (1 sub-exception)` — which says nothing
    about what actually went wrong. Walking into the group recovers the real cause.
    """
    while isinstance(exc, BaseExceptionGroup) and exc.exceptions:
        exc = exc.exceptions[0]
    label = type(exc).__name__
    message = str(exc).strip()
    return f"{label}: {message}" if message else label


def pending_question(detail: dict) -> str | None:
    """Return the user turn awaiting a reply in `detail`, or None.

    `send_message` records only the user's turn — replies come from us — so a transcript
    ending on a user message is by definition unanswered. A transcript ending on an `error`
    turn was already answered, with a failure; retrying it would loop.
    """
    messages = detail.get("messages", [])
    if not messages:
        return None
    last = messages[-1]
    return last["content"] if last.get("role") == "user" else None


async def report_error(agent: SentryAgent, space_id: int, exc: BaseException) -> None:
    """Write a failed turn into the chat so the user sees why, not a spinner.

    Best-effort: if posting the error itself fails, the app is almost certainly gone and the
    reconnect logic in `serve` is what should handle that.
    """
    summary = describe(exc)
    trace = "".join(traceback.format_exception(type(exc), exc, exc.__traceback__))
    print(f"[space {space_id}] failed: {summary}", file=sys.stderr)
    try:
        await agent.tools.call(
            "post_error_message",
            chat_space_id=space_id,
            message=summary,
            details=trace,
        )
    except Exception as post_failure:
        print(f"could not report the error: {describe(post_failure)}", file=sys.stderr)


def adopt_model(agent: SentryAgent, model: str | None) -> None:
    """Switch to the model the app's picker has selected."""
    if not model or model == agent.spec.key:
        return
    try:
        spec = agent.set_model(model, strict=True)
    except UnknownModelError as exc:
        # A model this build doesn't know. Keep answering on the current one.
        print(f"warning: {exc}", file=sys.stderr)
    else:
        print(f"model -> {spec.model_id} (selected in the app)")


async def answer(agent: SentryAgent, space_id: int) -> None:
    """Answer the outstanding question in one chat space, if there is one."""
    detail = await agent.tools.call("get_chat_space", id=space_id)
    question = pending_question(detail)
    if question is None:
        return

    print(f"[space {space_id}] {question}")

    async def on_step(step: str) -> None:
        # Best-effort: progress is a nicety, and failing to report it should never take down
        # the turn that is actually producing the answer.
        try:
            await agent.tools.call(
                "post_thinking_step", chat_space_id=space_id, step=step
            )
        except Exception:
            pass

    # A failure here is the model's or a tool's, not the app being gone: report it into the
    # chat and carry on. Letting it propagate would drop the connection and — worse — leave
    # this message with no reply at all, which the panel can only render as waiting forever.
    try:
        # thread_id ties this space to its own LangGraph conversation.
        reply = await agent.ask(question, thread_id=str(space_id), on_step=on_step)
    except asyncio.CancelledError:
        # The user hit stop. The app has already written the notice into the transcript, so
        # there is nothing to post — just let go of the work.
        print(f"[space {space_id}] interrupted")
        raise
    except Exception as exc:
        await report_error(agent, space_id, exc)
    else:
        await agent.tools.call(
            "post_assistant_message", chat_space_id=space_id, content=reply
        )
        print(f"[space {space_id}] replied ({len(reply)} chars)")


class Turns:
    """The agent turns currently in flight, one per chat space.

    Turns run as tasks rather than inline so the event socket keeps being read while a model
    call is outstanding. Awaiting a turn directly in the receive loop would mean a stop
    request could not arrive until the turn it is trying to stop had already finished.
    """

    def __init__(self, agent: SentryAgent) -> None:
        self._agent = agent
        self._tasks: dict[int, asyncio.Task] = {}

    def start(self, space_id: int) -> None:
        existing = self._tasks.get(space_id)
        if existing is not None and not existing.done():
            return  # already working on this space

        task = asyncio.create_task(answer(self._agent, space_id))
        self._tasks[space_id] = task
        task.add_done_callback(lambda _: self._tasks.pop(space_id, None))

    def cancel(self, space_id: int) -> None:
        task = self._tasks.get(space_id)
        if task is not None and not task.done():
            task.cancel()

    def cancel_all(self) -> None:
        for task in list(self._tasks.values()):
            if not task.done():
                task.cancel()

    async def drain(self) -> None:
        pending = [t for t in self._tasks.values() if not t.done()]
        if pending:
            await asyncio.gather(*pending, return_exceptions=True)


async def reconcile(agent: SentryAgent, turns: Turns) -> None:
    """Answer anything that came in while we weren't connected."""
    for space in await agent.tools.call("list_chat_spaces"):
        turns.start(space["id"])


async def handle(agent: SentryAgent, turns: Turns, event: dict) -> None:
    kind = event.get("type")
    if kind == "chat_message":
        turns.start(event["chat_space_id"])
    elif kind == "cancel_chat":
        turns.cancel(event["chat_space_id"])
    elif kind == "agent_config":
        adopt_model(agent, event.get("model"))


async def serve(agent: SentryAgent) -> None:
    """React to the app's events until it goes away.

    Exits once the socket has been unreconnectable for [`MAX_RECONNECT_ATTEMPTS`]. That is
    how this process avoids outliving the app that started it: the app kills us on a clean
    shutdown, but a crash or a force-kill skips that path entirely, and an orphan here would
    sit holding a connection to nothing.
    """
    print(f"connected to the app, answering with {agent.spec.model_id} — ctrl-c to stop")
    attempts = 0
    turns = Turns(agent)
    greeted = False

    while True:
        try:
            greeted = False
            async for event in stream():
                attempts = 0
                # The first agent_config of a connection is the greeting. Reconcile then,
                # not before: by that point the model is current, so a message left over
                # from before we connected is answered on the one the user selected.
                if event.get("type") == "agent_config" and not greeted:
                    greeted = True
                    adopt_model(agent, event.get("model"))
                    await reconcile(agent, turns)
                else:
                    await handle(agent, turns, event)
        except (OSError, websockets.exceptions.WebSocketException) as exc:
            attempts += 1
            print(
                f"disconnected ({attempts}/{MAX_RECONNECT_ATTEMPTS}): {describe(exc)}",
                file=sys.stderr,
            )
        else:
            # A clean close is the app shutting down; nothing to wait around for.
            attempts += 1
            print(f"connection closed ({attempts}/{MAX_RECONNECT_ATTEMPTS})", file=sys.stderr)

        if attempts >= MAX_RECONNECT_ATTEMPTS:
            print("app is gone — shutting down", file=sys.stderr)
            # Nothing in flight can be delivered anywhere, so drop it rather than let the
            # event loop shut down with tasks still pending.
            turns.cancel_all()
            await turns.drain()
            return
        await asyncio.sleep(RECONNECT_SECONDS)
