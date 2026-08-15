"""The model registry and how a name becomes a chat model.

Every model here is served by Groq. Swapping between them is a first-class operation rather
than an edit: pick one with `--model`, set `SYSTEM_EXPERT_AGENT_MODEL`, or switch at runtime with
`/model <name>` in the chat panel (see `chat_bridge`).

Adding a model on Groq is a one-line entry in `MODELS`. Adding a *different provider* means
giving `ModelSpec` a `provider` field and branching in `build_model` — kept out until it's
actually needed, since everything here shares one client and one API key.
"""

from __future__ import annotations

import os
from dataclasses import dataclass

from langchain_groq import ChatGroq

API_KEY_ENV = "GROQ_API_KEY"
MODEL_ENV = "SYSTEM_EXPERT_AGENT_MODEL"


@dataclass(frozen=True)
class ModelSpec:
    """One selectable model. `key` is the short name a user types."""

    key: str
    model_id: str
    description: str


# Ordered best-default-first. Verified against Groq's /v1/models — all three carry a 131k
# context window, which matters because tool results here can run to tens of thousands of
# tokens even after the Rust side's projection and downsampling.
MODELS: tuple[ModelSpec, ...] = (
    ModelSpec(
        key="gpt-oss",
        model_id="openai/gpt-oss-120b",
        # ASCII only in these descriptions: they print straight to a terminal, and a
        # Windows console on a legacy code page mangles or rejects non-ASCII.
        description="OpenAI GPT-OSS 120B - the default; strongest tool use of the three",
    ),
    ModelSpec(
        key="gpt-oss-20b",
        model_id="openai/gpt-oss-20b",
        description="OpenAI GPT-OSS 20B - smaller and faster, weaker at multi-step tool calls",
    ),
    ModelSpec(
        key="qwen",
        model_id="qwen/qwen3.6-27b",
        description="Qwen3.6 27B",
    ),
    ModelSpec(
        key="llama",
        model_id="llama-3.3-70b-versatile",
        description="Llama 3.3 70B Versatile",
    ),
)

DEFAULT_MODEL_KEY = MODELS[0].key

_BY_KEY = {spec.key: spec for spec in MODELS}


class UnknownModelError(LookupError):
    """Raised for a model name that isn't in the registry."""

    def __init__(self, name: str) -> None:
        options = ", ".join(spec.key for spec in MODELS)
        super().__init__(f"unknown model {name!r} - choose one of: {options}")
        self.name = name


def resolve(name: str | None = None, *, strict: bool = False) -> ModelSpec:
    """Resolve a model name to its spec.

    Precedence: explicit argument, then `SYSTEM_EXPERT_AGENT_MODEL`, then the default. Accepts a
    registry key (`qwen`) or a full Groq model id (`qwen/qwen3.6-27b`).

    `strict` rejects anything not in the registry. Use it for names arriving from the
    desktop app, which only ever sends registry keys: the lenient path below would accept a
    stale or misspelled name as a raw model id, and the failure would then surface as a 404
    from Groq on every subsequent turn instead of at the point of the mistake.
    """
    requested = name or os.environ.get(MODEL_ENV) or DEFAULT_MODEL_KEY

    if requested in _BY_KEY:
        return _BY_KEY[requested]

    by_id = next((spec for spec in MODELS if spec.model_id == requested), None)
    if by_id is not None:
        return by_id

    # An unregistered id is allowed through for hand-driven use, so a model Groq adds
    # tomorrow works without waiting for the registry — but only if it looks like a model id
    # rather than a typo'd key, so `--model qwem` still fails loudly.
    if not strict and "/" in requested:
        return ModelSpec(key=requested, model_id=requested, description="(not in registry)")

    raise UnknownModelError(requested)


def build_model(spec: ModelSpec, **kwargs) -> ChatGroq:
    """Instantiate the chat model for `spec`.

    Raises if the API key is missing, rather than letting the failure surface later as an
    opaque auth error in the middle of an agent turn.
    """
    if not os.environ.get(API_KEY_ENV):
        raise RuntimeError(
            f"{API_KEY_ENV} is not set — put it in agent/.env or export it"
        )
    # temperature=0: this agent reports machine state and can terminate processes, so
    # reproducibility matters more than variety.
    return ChatGroq(model=spec.model_id, temperature=0, **kwargs)
