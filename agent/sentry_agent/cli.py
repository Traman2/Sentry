"""Command-line entry point."""

from __future__ import annotations

import argparse
import asyncio
import sys

from dotenv import load_dotenv

from .agent import SentryAgent
from .chat_bridge import describe, serve
from .config import MCP_URL
from .models import MODELS, MODEL_ENV, UnknownModelError


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="sentry-agent",
        description="LangGraph agent for the Sentry desktop app's MCP tools.",
    )
    action = parser.add_mutually_exclusive_group(required=True)
    action.add_argument(
        "--list-tools", action="store_true", help="list the MCP tools and exit"
    )
    action.add_argument(
        "--list-models", action="store_true", help="list selectable models and exit"
    )
    action.add_argument("--ask", metavar="QUESTION", help="run one agent turn")
    action.add_argument(
        "--serve", action="store_true", help="answer chat messages from the app"
    )
    parser.add_argument(
        "--model",
        metavar="NAME",
        help=f"model to use — a registry key or a Groq model id (env: {MODEL_ENV})",
    )
    parser.add_argument("--url", help=f"override the MCP URL (default {MCP_URL})")
    return parser


def print_models(current: str) -> None:
    print("models:\n")
    for spec in MODELS:
        marker = "*" if spec.key == current else " "
        print(f" {marker} {spec.key:<12} {spec.model_id:<26} {spec.description}")
    print("\n* = current. Any other Groq model id also works.")
    print("When the desktop app runs the agent, its chat model picker overrides --model.")


def _force_utf8_output() -> None:
    """Stop model output from crashing the process on a legacy Windows console.

    A Windows console defaults to a code page like cp1252, and model replies routinely
    contain characters it cannot encode — a non-breaking hyphen is enough to raise
    UnicodeEncodeError and kill the run after the work is already done. Reconfiguring with
    `errors="replace"` degrades an unencodable character to `?` instead.

    `line_buffering` is set for the same practical reason: `--serve` is a long-running poll
    loop, and redirected to a log its progress would otherwise sit in a block buffer for
    minutes at a time.
    """
    for stream in (sys.stdout, sys.stderr):
        reconfigure = getattr(stream, "reconfigure", None)
        if reconfigure is not None:
            try:
                reconfigure(encoding="utf-8", errors="replace", line_buffering=True)
            except (OSError, ValueError):
                pass  # redirected to something that can't be reconfigured; not fatal


async def run(argv: list[str] | None = None) -> int:
    _force_utf8_output()
    # The key normally lives in agent/.env rather than the shell environment.
    load_dotenv()

    args = build_parser().parse_args(argv)

    try:
        agent = SentryAgent(args.url or MCP_URL, model=args.model)
    except UnknownModelError as exc:
        print(exc, file=sys.stderr)
        return 2

    try:
        if args.list_models:
            print_models(agent.spec.key)
            return 0

        try:
            tools = await agent.load_tools()
        except Exception as exc:
            print(
                f"could not reach the Sentry MCP server at {agent.url}: {describe(exc)}",
                file=sys.stderr,
            )
            print("is the desktop app running?", file=sys.stderr)
            return 1

        if args.list_tools:
            print(f"{len(tools)} tools at {agent.url}\n")
            for tool in sorted(tools, key=lambda t: t.name):
                summary = (tool.description or "").split(".")[0].strip()
                print(f"  {tool.name:<26} {summary}")
            return 0

        try:
            if args.ask:
                print(await agent.ask(args.ask))
                return 0
            await serve(agent)
            return 0
        except RuntimeError as exc:  # missing API key, surfaced by models.build_model
            print(exc, file=sys.stderr)
            return 1
    finally:
        # Releases the checkpointer's sqlite connection. A no-op if the graph was never
        # built, e.g. --list-tools / --list-models exiting above before any turn ran.
        await agent.aclose()


def main(argv: list[str] | None = None) -> int:
    try:
        return asyncio.run(run(argv))
    except KeyboardInterrupt:
        return 130
