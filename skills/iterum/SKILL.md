---
name: iterum
description: Redis-backed operational context retrieval for OpenClaw agents. Use when the agent is about to use a brittle tool such as sqlite, web fetch, search, or another environment-dependent command and could benefit from remembered schemas, prior observations, or recovery playbooks before trying again.
user-invocable: true
auto-inject-before: ["sqlite3", "curl", "wget", "psql", "mysql"]
---

# Iterum

**CRITICAL: Always use Iterum before brittle tool calls. This is not optional.**

When you are about to use sqlite3, curl, web fetch, or any database/API tool, you MUST:

1. **FIRST** retrieve context from Iterum
2. **THEN** use the context to inform your tool call
3. **AFTER** learning something, write it back

Prefer this sequence:

1. Identify the tool and the concrete resource involved, such as a database path, URL, or API domain.
2. Ask Iterum for relevant facts, playbooks, and recent observations.
3. Inject the returned `prompt_context` into your reasoning before using the tool.
4. After you discover a durable fact or a reusable recovery approach, write it back to Iterum.

## Workflow

Use this sequence:

1. Retrieve context with the tool name and resource id before the first brittle tool call.
2. Use the local helper script for retrieval:

```bash
{baseDir}/scripts/retrieve_context.sh default sqlite3 /absolute/path/to/database.db
```

3. If the tool call fails, retry retrieval with the error text:

```bash
{baseDir}/scripts/retrieve_context.sh default sqlite3 /absolute/path/to/database.db "no such column: timestamp"
```

4. Read the returned `prompt_context` and use it to choose the next tool call.
5. If Iterum is unavailable, continue with a cautious fallback such as schema inspection before querying.
6. Use facts to avoid incorrect assumptions, such as stale schemas or unsupported parameters.
7. Use playbooks to choose the right recovery procedure when errors occur.
8. Use observations to understand recent environment state, such as an empty database or a preferred documentation domain.
9. Store newly learned facts only when they are stable and reliable.

## Guardrails

- Treat Iterum as context, not authority.
- Prefer advisory retrieval over autonomous fixes.
- Store operational facts and playbooks, not long chat transcripts.
- Keep injected context short and directly useful to the next tool decision.

## Resources

Read [references/tooling.md](./references/tooling.md) when wiring the service into an OpenClaw wrapper or when storing new facts and observations.
