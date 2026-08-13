"""The push channel from the desktop app.

The app announces things over a WebSocket rather than making us ask: a user sent a message,
the model picker changed. The socket also stands in for a liveness check — it closes the
moment the app dies, including on a crash or force-kill, which is a faster and far more
definite signal than watching requests fail.
"""

from __future__ import annotations

import json
from collections.abc import AsyncIterator
from typing import Any

import websockets

from .config import EVENTS_URL


async def stream(url: str = EVENTS_URL) -> AsyncIterator[dict[str, Any]]:
    """Connect and yield each event until the socket closes.

    Raises on a failed connect and returns cleanly on close, leaving the caller to decide
    whether to reconnect or give up — see `chat_bridge.serve`.
    """
    async with websockets.connect(url) as socket:
        async for raw in socket:
            try:
                event = json.loads(raw)
            except json.JSONDecodeError:
                continue  # not ours to interpret; ignore rather than tear down the socket
            if isinstance(event, dict) and "type" in event:
                yield event
