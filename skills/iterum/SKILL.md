---
name: iterum
description: Redis-backed operational context retrieval for OpenClaw agents. Use when the agent is about to use a brittle tool such as sqlite, web fetch, search, or another environment-dependent command and could benefit from remembered schemas, prior observations, or recovery playbooks before trying again.
---

# Iterum

Use Iterum before brittle tool calls when missing context is likely to cause avoidable errors.

Prefer this sequence:

1. Identify the tool and the concrete resource involved, such as a database path, URL, or API domain.
2. Ask Iterum for relevant facts, playbooks, and recent observations.
3. Inject the returned `prompt_context` into your reasoning before using the tool.
4. After you discover a durable fact or a reusable recovery approach, write it back to Iterum.

## Workflow

Use this sequence:

1. Retrieve context with the tool name and resource id before the first brittle tool call.
2. Use facts to avoid incorrect assumptions, such as stale schemas or unsupported parameters.
3. Use playbooks to choose the right recovery procedure when errors occur.
4. Use observations to understand recent environment state, such as an empty database or a preferred documentation domain.
5. Store newly learned facts only when they are stable and reliable.

## Guardrails

- Treat Iterum as context, not authority.
- Prefer advisory retrieval over autonomous fixes.
- Store operational facts and playbooks, not long chat transcripts.
- Keep injected context short and directly useful to the next tool decision.

## Resources

Read [references/tooling.md](./references/tooling.md) when wiring the service into an OpenClaw wrapper or when storing new facts and observations.
