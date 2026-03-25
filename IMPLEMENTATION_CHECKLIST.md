# Iterum Implementation Checklist

## Goal

Build Iterum as a Python service that retrieves and stores operational context for OpenClaw agents. The first version should improve agent behavior by injecting relevant facts, playbooks, and recent observations before brittle tool use.

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
  app/
    __init__.py
    main.py
    config.py
    models.py
    store.py
    retrieval.py
    ranking.py
    context_formatter.py
    adapters/
      __init__.py
      sqlite.py
      web.py
    routes/
      __init__.py
      context.py
      debug.py
  skills/
    iterum/
      SKILL.md
      agents/openai.yaml
      references/tooling.md
  tests/
    test_context_retrieval.py
    test_sqlite_adapter.py
    test_web_adapter.py
  PROJECT_PLAN.md
  IMPLEMENTATION_CHECKLIST.md
```

## Exact Files To Build

### `app/main.py`

Responsibility:
- create FastAPI app
- register routes
- initialize Redis-backed store
- expose dependency injection helpers

Interfaces:
- app factory or module-level `app`
- `get_store()`
- `get_retriever()`

### `app/config.py`

Responsibility:
- load environment variables
- define service settings

Fields:
- `ITERUM_REDIS_URL`
- `ITERUM_MEMORY_TTL_SECONDS`
- `ITERUM_DEFAULT_NAMESPACE`
- `ITERUM_MAX_CONTEXT_ITEMS`

### `app/models.py`

Responsibility:
- define Pydantic request and response models

Required models:
- `ContextRetrieveRequest`
- `ContextRetrieveResponse`
- `FactRecord`
- `PlaybookRecord`
- `ObservationRecord`
- `StoreFactRequest`
- `StorePlaybookRequest`
- `StoreObservationRequest`

Suggested shapes:

```python
class ContextRetrieveRequest(BaseModel):
    namespace: str = "default"
    user_id: str | None = None
    tool_name: str
    resource_id: str | None = None
    task_type: str | None = None
    error_text: str | None = None
    query: str | None = None
```

```python
class ContextRetrieveResponse(BaseModel):
    namespace: str
    tool_name: str
    resource_id: str | None
    facts: list[FactRecord]
    playbooks: list[PlaybookRecord]
    observations: list[ObservationRecord]
    prompt_context: str
```

### `app/store.py`

Responsibility:
- read and write Redis records
- maintain indexes
- expose high-level storage methods

Required methods:
- `put_fact(record)`
- `put_playbook(record)`
- `put_observation(record)`
- `list_facts(namespace, tool_name, resource_id=None)`
- `list_playbooks(namespace, tool_name, error_family=None)`
- `list_observations(namespace, tool_name, resource_id=None)`

Redis key design:
- `iterum:fact:{namespace}:{tool_name}:{resource_id}:{fact_id}`
- `iterum:playbook:{namespace}:{tool_name}:{playbook_id}`
- `iterum:observation:{namespace}:{tool_name}:{resource_id}:{observation_id}`

Optional index keys:
- `iterum:index:facts:{namespace}:{tool_name}:{resource_id}`
- `iterum:index:playbooks:{namespace}:{tool_name}`
- `iterum:index:observations:{namespace}:{tool_name}:{resource_id}`

### `app/retrieval.py`

Responsibility:
- retrieve candidate records from the store
- delegate ranking
- return structured context response

Required interface:

```python
class ContextRetriever:
    def retrieve(self, request: ContextRetrieveRequest) -> ContextRetrieveResponse:
        ...
```

Behavior:
- fetch facts by exact tool and resource match
- fetch playbooks by tool and optional error family
- fetch recent observations by tool and resource
- pass results to ranking and formatting layers

### `app/ranking.py`

Responsibility:
- rank retrieved items so the prompt stays short and useful

Rules for v1:
- exact resource match first
- higher confidence first
- fresher observations first
- max items per category from config

Required functions:
- `rank_facts(...)`
- `rank_playbooks(...)`
- `rank_observations(...)`

### `app/context_formatter.py`

Responsibility:
- turn retrieved records into prompt-ready context text

Required function:
- `format_prompt_context(facts, playbooks, observations) -> str`

Output style:
- concise
- operational
- no extra narration

Example output:

```text
Relevant facts:
- SQLite table `strategy_pnl` has no `timestamp` column.

Recommended playbooks:
- On SQLite unknown-column errors, inspect `.schema <table>` before retrying.

Recent observations:
- `bot_btc_directional_dry.db` currently has zero rows in `runs`, `fills`, `orders`, and `positions`.
```

### `app/adapters/sqlite.py`

Responsibility:
- normalize SQLite-specific context
- define helper functions for schema and empty-state observations

Required helper functions:
- `make_schema_fact(resource_id, table_name, columns, summary)`
- `make_empty_state_observation(resource_id, counts)`
- `make_unknown_column_playbook()`

First facts/playbooks to support:
- schema summaries
- "inspect `.tables` and `.schema` before retrying unknown queries"
- "if core tables are empty, report empty DB state early"

### `app/adapters/web.py`

Responsibility:
- normalize web/search-specific context

Required helper functions:
- `make_search_then_fetch_playbook()`
- `make_preferred_domain_fact(topic, domain)`
- `make_fetch_observation(resource_id, note)`

First playbooks to support:
- if input is not a valid URL, search first
- prefer official docs domains when known

### `app/routes/context.py`

Responsibility:
- expose public context endpoints

Endpoints:
- `POST /v1/context/retrieve`
- `POST /v1/context/facts`
- `POST /v1/context/playbooks`
- `POST /v1/context/observations`

### `app/routes/debug.py`

Responsibility:
- expose inspection endpoints for manual verification

Endpoints:
- `GET /health`
- `GET /v1/debug/facts`
- `GET /v1/debug/playbooks`
- `GET /v1/debug/observations`

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

Even before deep OpenClaw integration, support a simple wrapper call:

```python
context = iterum.retrieve(
    tool_name="sqlite3",
    resource_id=db_path,
    task_type="analysis",
    error_text=recent_error,
)
```

Then prepend:

```text
Before using sqlite3, consider this known context:
{context.prompt_context}
```

## Tests To Add

### `tests/test_context_retrieval.py`

Verify:
- exact tool and resource facts are returned
- prompt context includes fact, playbook, and observation sections
- ranking prefers exact matches

### `tests/test_sqlite_adapter.py`

Verify:
- schema fact helpers create correct record shapes
- unknown-column playbook is formatted correctly
- empty-state observation is generated correctly

### `tests/test_web_adapter.py`

Verify:
- search-then-fetch playbook is generated
- preferred domain facts rank correctly

## Build Order

### Phase 1: Service foundation

- [ ] Replace the current demo-specific models with generic context models in `app/models.py`
- [ ] Replace mistake-memory-specific store logic in `app/store.py` with fact, playbook, and observation storage
- [ ] Create `app/retrieval.py`
- [ ] Create `app/ranking.py`
- [ ] Create `app/context_formatter.py`
- [ ] Split route handlers into `app/routes/context.py` and `app/routes/debug.py`
- [ ] Update `app/main.py` to wire new routes and dependencies

### Phase 2: SQLite first-class support

- [ ] Create `app/adapters/sqlite.py`
- [ ] Seed at least one SQLite schema fact
- [ ] Seed at least one SQLite recovery playbook
- [ ] Support recent empty-state observations
- [ ] Add SQLite-focused retrieval tests

### Phase 3: Web support

- [ ] Create `app/adapters/web.py`
- [ ] Add search-then-fetch playbook support
- [ ] Add preferred domain facts
- [ ] Add web retrieval tests

### Phase 4: OpenClaw packaging

- [ ] Update `skills/iterum/SKILL.md` to reflect context-first behavior
- [ ] Update `skills/iterum/references/tooling.md` with retrieval/store examples
- [ ] Validate the skill metadata and wording

### Phase 5: Visibility and deployment

- [ ] Add debug endpoints for listing stored records
- [ ] Add a CLI or curl examples in `README.md`
- [ ] Run local Redis and verify manual end-to-end retrieval
- [ ] Deploy the FastAPI app
- [ ] Connect to Redis Cloud

## Deferred Work

- Contextual-assisted clustering of similar observations
- Automatic extraction of facts from run traces
- Auto-refresh of stale observations
- Per-user personalization beyond namespace scoping
- Read-only preflight automation for selected low-risk tools
