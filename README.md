# Iterum

Iterum is a Redis-backed context layer for OpenClaw. It stores operational facts, recovery playbooks, and recent observations about brittle tools, then returns prompt-ready context before the agent retries the same class of task.

Built with Rust using Axum, Tokio, and serde.

## Scope

- Redis is the durable memory layer
- Axum exposes the service endpoints
- The service returns prompt-ready context bundles for agents and wrappers
- SQLite and web helpers prove the architecture is general
- An OpenClaw skill tells agents when to query Iterum before brittle tool calls
- `memory://` is supported for local development when Redis is not available
- Tool-result event ingestion automatically extracts facts, constraints, and failure patterns

## Run locally

```bash
cargo run
```

Or with environment overrides:

```bash
ITERUM_PORT=8000 ITERUM_REDIS_URL=memory://local cargo run
```

## Build

```bash
cargo build --release
```

The binary is at `target/release/iterum`.

## Test

```bash
cargo test
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

```bash
curl -s http://127.0.0.1:8000/v1/events/tool-result \
  -H 'content-type: application/json' \
  -d '{
    "namespace": "default",
    "tool_name": "bash",
    "command": "sqlite3 ~/test.db '\''.schema orders'\''",
    "resource_id": "sqlite:~/test.db",
    "stdout": "CREATE TABLE orders (id INTEGER PRIMARY KEY, amount REAL, created_at TEXT);",
    "is_error": false
  }' | jq
```

The retrieval response includes `facts`, `playbooks`, `observations`, `constraints`, `failure_patterns`, and a `prompt_context` block ready to inject into the agent prompt.

## API endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/v1/context/retrieve` | Retrieve context for a tool + resource |
| POST | `/v1/context/facts` | Store a fact |
| POST | `/v1/context/playbooks` | Store a playbook |
| POST | `/v1/context/observations` | Store an observation |
| POST | `/v1/events/tool-result` | Ingest tool output, auto-extract facts/constraints/patterns |
| GET | `/health` | Health check |
| GET | `/v1/debug/facts` | List stored facts |
| GET | `/v1/debug/playbooks` | List stored playbooks |
| GET | `/v1/debug/observations` | List stored observations |
| GET | `/v1/debug/constraints` | List stored constraints |
| GET | `/v1/debug/failure-patterns` | List stored failure patterns |

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ITERUM_REDIS_URL` | `redis://localhost:6379/0` | Redis URL or `memory://local` for in-memory |
| `ITERUM_PORT` | `8000` | Server listen port |
| `ITERUM_MEMORY_TTL_SECONDS` | `604800` (7 days) | Default TTL for observations/constraints |
| `ITERUM_DEFAULT_NAMESPACE` | `default` | Default namespace |
| `ITERUM_MAX_CONTEXT_ITEMS` | `3` | Max items per category in retrieval |

## Local backend modes

- `ITERUM_REDIS_URL=memory://local`
  - use the built-in in-memory store
  - best for local development and smoke tests
- `ITERUM_REDIS_URL=redis://host:port/db`
  - use a real Redis instance
  - best for shared or deployed memory

## Project structure

```
src/
  main.rs              # Entry point
  lib.rs               # Module exports
  config.rs            # Environment configuration
  models.rs            # Record types and request/response models
  error.rs             # Error types
  retrieval.rs         # Context retrieval pipeline
  ranking.rs           # Relevance ranking
  formatter.rs         # Prompt context formatting
  store/
    mod.rs             # Store trait
    memory.rs          # In-memory store implementation
    redis_store.rs     # Redis store implementation
  routes/
    mod.rs             # Router setup
    context.rs         # Context CRUD endpoints
    events.rs          # Tool-result event ingestion
    debug.rs           # Debug/inspection endpoints
  extractors/
    mod.rs             # Extractor registry
    sqlite.rs          # SQLite output extractor
    web.rs             # Web/CLI output extractor
    cli.rs             # CLI output extractor
tests/
  integration.rs       # Integration tests
skills/
  iterum/              # OpenClaw skill definition
```
