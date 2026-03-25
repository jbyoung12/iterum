# Iterum

Iterum is a Redis-backed context layer for OpenClaw. It stores operational facts, recovery playbooks, and recent observations about brittle tools, then returns prompt-ready context before the agent retries the same class of task.

## Scope

- Redis is the durable memory layer
- FastAPI exposes the service endpoints
- The service returns prompt-ready context bundles for agents and wrappers
- SQLite and web helpers prove the architecture is general
- An OpenClaw skill tells agents when to use Iterum before brittle tool calls
- `memory://` is supported for local development when Redis is not available

## Run locally

```bash
cp .env.example .env
uv venv
uv pip install -e ".[dev]"
uvicorn app.main:app --reload
```

## Example flow

```bash
curl -s http://127.0.0.1:8000/v1/context/facts \
  -H 'content-type: application/json' \
  -d '{
    "namespace": "default",
    "tool_name": "sqlite3",
    "resource_id": "/tmp/demo.db",
    "topic": "schema",
    "title": "strategy_pnl schema",
    "content": "Columns: id, strategy, realized_pnl, unrealized_pnl, total_pnl, created_at",
    "confidence": 1.0
  }' | jq
```

```bash
curl -s http://127.0.0.1:8000/v1/context/playbooks \
  -H 'content-type: application/json' \
  -d '{
    "namespace": "default",
    "tool_name": "sqlite3",
    "error_family": "unknown_column",
    "title": "Inspect schema before retrying",
    "steps": [
      "Run .tables",
      "Run .schema <table> for the referenced table",
      "Rewrite the query only after verifying columns"
    ],
    "confidence": 0.95
  }' | jq
```

```bash
curl -s http://127.0.0.1:8000/v1/context/retrieve \
  -H 'content-type: application/json' \
  -d '{
    "namespace": "default",
    "tool_name": "sqlite3",
    "resource_id": "/tmp/demo.db",
    "task_type": "analysis",
    "error_text": "no such column: timestamp"
  }' | jq
```

The retrieval response includes `facts`, `playbooks`, `observations`, and a `prompt_context` block ready to inject into the agent prompt.

## Local backend modes

- `ITERUM_REDIS_URL=memory://local`
  - use the built-in in-memory store
  - best for local development and smoke tests
- `ITERUM_REDIS_URL=redis://host:port/db`
  - use a real Redis instance
  - best for shared or deployed memory
