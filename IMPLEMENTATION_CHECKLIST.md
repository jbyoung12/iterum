# Iterum Implementation Checklist

## Goal

Build Iterum as a Rust service (Axum + Tokio) that retrieves and stores operational context for OpenClaw agents. The first version should improve agent behavior by injecting relevant facts, playbooks, and recent observations before brittle tool use.

## Core Runtime Flow

1. Agent or wrapper prepares a tool task.
2. Wrapper calls Iterum retrieval API with tool, resource, and task metadata.
3. Iterum returns a compact context bundle:
   - facts
   - playbooks
   - observations
4. Wrapper injects that bundle into the LLM or tool prompt.
5. Agent executes the task.
6. If the run discovers new useful information, wrapper calls Iterum store or observe APIs.
7. Future similar tasks retrieve that knowledge earlier.

## Target Repo Structure

```text
iterum/
  src/
    main.rs
    lib.rs
    config.rs
    models.rs
    error.rs
    retrieval.rs
    ranking.rs
    formatter.rs
    store/
      mod.rs
      memory.rs
      redis_store.rs
    routes/
      mod.rs
      context.rs
      events.rs
      debug.rs
    extractors/
      mod.rs
      sqlite.rs
      web.rs
      cli.rs
  skills/
    iterum/
      SKILL.md
      references/tooling.md
  tests/
    integration.rs
  Cargo.toml
  PROJECT_PLAN.md
  IMPLEMENTATION_CHECKLIST.md
```

## Exact Files Built

### `src/main.rs`
- Creates Axum app, registers routes, initializes store, starts Tokio server

### `src/config.rs`
- Loads environment variables: `ITERUM_REDIS_URL`, `ITERUM_MEMORY_TTL_SECONDS`, `ITERUM_DEFAULT_NAMESPACE`, `ITERUM_MAX_CONTEXT_ITEMS`, `ITERUM_PORT`

### `src/models.rs`
- Defines serde record types: `FactRecord`, `PlaybookRecord`, `ObservationRecord`, `ConstraintRecord`, `FailurePatternRecord`
- Request/response types: `ContextRetrieveRequest`, `ContextRetrieveResponse`, `StoreFactRequest`, `StorePlaybookRequest`, `StoreObservationRequest`, `ToolResultEvent`, `ToolResultResponse`

### `src/store/mod.rs`
- Defines the `Store` trait with async methods for all record types

### `src/store/memory.rs`
- In-memory `HashMap`-based store with TTL expiration for observations, constraints, and failure patterns
- FIFO eviction for failure patterns per resource

### `src/store/redis_store.rs`
- Redis-backed store implementation

### `src/retrieval.rs`
- `ContextRetriever` generic over `Store`, fetches and ranks facts/playbooks/observations/constraints/failure patterns

### `src/ranking.rs`
- Ranks items by confidence and freshness, limits per category

### `src/formatter.rs`
- Formats retrieved records into prompt-ready context text

### `src/extractors/`
- `sqlite.rs`: Extracts schema facts, constraints, and failure patterns from SQLite tool output
- `web.rs`: Web output extractor
- `cli.rs`: CLI output extractor

### `src/routes/context.rs`
- `POST /v1/context/retrieve`, `/facts`, `/playbooks`, `/observations`

### `src/routes/events.rs`
- `POST /v1/events/tool-result` — auto-extracts facts/constraints/patterns from tool output

### `src/routes/debug.rs`
- `GET /health`, `/v1/debug/facts`, `/playbooks`, `/observations`, `/constraints`, `/failure-patterns`

## HTTP API Contracts

### `POST /v1/context/retrieve`

Request:

```json
{
  "namespace": "default",
  "user_id": "josh",
  "tool_name": "sqlite3",
  "resource_id": "/Users/joshuayoung/code/drachma/bot_btc_directional_dry.db",
  "task_type": "analyze_performance",
  "error_text": "no such column: timestamp",
  "query": "Investigate why btc_directional dry run is performing badly"
}
```

Response:

```json
{
  "namespace": "default",
  "tool_name": "sqlite3",
  "resource_id": "/Users/joshuayoung/code/drachma/bot_btc_directional_dry.db",
  "facts": [],
  "playbooks": [],
  "observations": [],
  "prompt_context": "Relevant facts:\n..."
}
```

### `POST /v1/context/facts`

Purpose:
- store durable facts like schema summaries or API argument shapes

### `POST /v1/context/playbooks`

Purpose:
- store reusable recovery guidance

### `POST /v1/context/observations`

Purpose:
- store recent environment state that may expire or change

## Redis Record Shapes

### Fact

```json
{
  "id": "strategy_pnl_schema_v1",
  "namespace": "default",
  "tool_name": "sqlite3",
  "resource_id": "/Users/joshuayoung/code/drachma/bot_btc_directional_dry.db",
  "topic": "schema",
  "title": "strategy_pnl schema",
  "content": "Columns: id, strategy, realized_pnl, unrealized_pnl, total_pnl, created_at",
  "confidence": 1.0,
  "updated_at": "2026-03-25T22:00:00Z"
}
```

### Playbook

```json
{
  "id": "sqlite_unknown_column_inspect_schema",
  "namespace": "default",
  "tool_name": "sqlite3",
  "error_family": "unknown_column",
  "title": "Inspect schema before retrying",
  "steps": [
    "Run `.tables`",
    "Run `.schema <table>` for the referenced table",
    "Rewrite the query only after verifying columns"
  ],
  "confidence": 0.95,
  "updated_at": "2026-03-25T22:00:00Z"
}
```

### Observation

```json
{
  "id": "dry_db_empty_state",
  "namespace": "default",
  "tool_name": "sqlite3",
  "resource_id": "/Users/joshuayoung/code/drachma/bot_btc_directional_dry.db",
  "topic": "empty_state",
  "content": "Current snapshot: runs=0, fills=0, orders=0, positions=0",
  "confidence": 0.9,
  "updated_at": "2026-03-25T22:00:00Z",
  "ttl_seconds": 86400
}
```

## OpenClaw Integration

### Skill behavior

The `skills/iterum/SKILL.md` instructions should tell the agent:
- query Iterum before brittle tool use
- inject `prompt_context` into reasoning
- write back durable facts and observations after learning something reliable

### First wrapper contract

Even before deep OpenClaw integration, support a simple wrapper call via HTTP:

```bash
curl -s http://127.0.0.1:8000/v1/context/retrieve \
  -H 'content-type: application/json' \
  -d '{"tool_name": "sqlite3", "resource_id": "/path/to/db", "task_type": "analysis", "error_text": "recent error"}'
```

Then prepend the returned `prompt_context` to the agent prompt.

## Tests

### `tests/integration.rs`

Verifies:
- Health check returns OK
- Store and retrieve facts round-trip
- Tool-result events extract schema facts
- Tool-result events detect errors and create constraints + failure patterns
- Failure pattern occurrence count increments on repeat
- Debug endpoints return stored records

## Build Order

### Phase 1: Service foundation (done)

- [x] Define serde models in `src/models.rs`
- [x] Implement `Store` trait and in-memory backend in `src/store/`
- [x] Create `src/retrieval.rs`
- [x] Create `src/ranking.rs`
- [x] Create `src/formatter.rs`
- [x] Wire routes in `src/routes/`
- [x] Entry point in `src/main.rs`

### Phase 2: SQLite first-class support (done)

- [x] Create `src/extractors/sqlite.rs`
- [x] Auto-extract schema facts from `.schema` output
- [x] Auto-create constraints from column errors
- [x] Track failure patterns with occurrence counts
- [x] Add integration tests

### Phase 3: Web/CLI support (done)

- [x] Create `src/extractors/web.rs`
- [x] Create `src/extractors/cli.rs`

### Phase 4: OpenClaw packaging (done)

- [x] Update `skills/iterum/SKILL.md` with context-first behavior
- [x] Update `skills/iterum/references/tooling.md`

### Phase 5: Visibility and deployment

- [x] Add debug endpoints for all record types
- [x] Add curl examples in `README.md`
- [ ] Deploy the Rust binary
- [ ] Connect to Redis Cloud

## Deferred Work

- Contextual-assisted clustering of similar observations
- Automatic extraction of facts from run traces
- Auto-refresh of stale observations
- Per-user personalization beyond namespace scoping
- Read-only preflight automation for selected low-risk tools
