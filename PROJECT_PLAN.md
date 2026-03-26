# Iterum Project Plan

## Project Description

Iterum is a context layer for OpenClaw that helps agents avoid repeating tool mistakes by remembering operational knowledge across runs. Instead of trying to auto-fix errors on its own, Iterum stores useful facts the agent discovered before, like database schemas, tool usage patterns, resource state, and proven recovery steps, in Redis and injects that context back into the agent when a similar task comes up again.

In practice, Iterum sits alongside tool use. Before an agent queries a SQLite database, fetches a web page, or calls another brittle tool, Iterum retrieves the most relevant facts and playbooks for that tool and resource, such as known table schemas, "inspect `.schema` before retrying," or "this DB is currently empty." That gives the LLM the context it was missing the first time, so it can reason correctly earlier, fail less, and recover faster. Over time, Iterum turns one-run debugging into durable operational memory for OpenClaw agents.

## Build Plan

1. Define the core product boundary.
   Iterum should be a Rust service for OpenClaw that retrieves and stores operational context, not a general autonomous fixer. The first supported outputs should be prompt-ready facts and recovery playbooks tied to a tool and resource.

2. Define the memory model in Redis.
   Create three first-class record types:
   - `facts`: durable context like DB schema, tool argument shapes, resource metadata
   - `playbooks`: reusable recovery guidance like "inspect schema before retrying"
   - `observations`: recent learned state like "this DB currently has zero runs/fills/orders"
   Add a small retrieval index keyed by tool family, resource identifier, and topic.

3. Define the runtime integration point with OpenClaw.
   Decide how Iterum is invoked before tool use. The first version should be advisory:
   - agent/tool wrapper asks Iterum for relevant context
   - Iterum returns compact context blocks
   - that context is injected into the LLM/tool call prompt
   This avoids patching agent internals too early.

4. Build the service skeleton.
   Use Axum, serde, and Redis. Implement endpoints like:
   - `POST /v1/context/retrieve`
   - `POST /v1/context/store`
   - `POST /v1/context/observe`
   - `GET /v1/context/debug`
   Keep the API generic across SQLite, web, and future adapters.

5. Implement the first retrieval pipeline.
   Input should include:
   - `tool_name`
   - `resource_id`
   - `task_type`
   - optional recent error text
   Retrieval should rank and return:
   - relevant facts
   - relevant playbooks
   - recent observations
   formatted as a short context bundle for the agent.

6. Implement the first learning pipeline.
   After a run, let Iterum store:
   - newly discovered schema/facts
   - successful recovery playbooks
   - recent environment observations
   Start with explicit writes from wrappers or scripts rather than trying to infer everything automatically.

7. Build the first concrete adapter: SQLite.
   This should support:
   - storing known schemas
   - storing table/column summaries
   - storing playbooks for query failures
   - storing observations like empty-table counts
   This is the strongest first use case because it directly addresses the failure pattern that motivated the project.

8. Build the second adapter: web/search.
   Support context like:
   - "if input is not a URL, search first"
   - preferred domains for known docs
   - known useful sources for recurring topics
   This proves the architecture is general and not just database-specific.

9. Add OpenClaw skill packaging.
   Create an `iterum` skill that instructs the agent when to query Iterum before using brittle tools and when to write back facts or observations after resolving an issue.

10. Add evaluation and visibility.
    Track:
    - number of repeated failures avoided
    - number of context hits
    - number of learned facts/playbooks reused
    - step reduction across similar tasks
    Add a simple debug view or CLI so stored context can be inspected easily.

11. Add safe automation later.
    After advisory mode works, optionally allow low-risk automatic preflight behavior for selected tools, like recommending or automatically running schema inspection for SQLite reads. Keep this off by default until retrieval quality is proven.

12. Add richer inference later.
    In the future, use Contextual only for:
    - clustering similar failures
    - generating compact summaries of learned context
    - classifying new observations into fact/playbook categories
    Do not make it the source of truth for stored operational memory.

## Immediate Build Order

1. Service skeleton
2. Redis schema
3. SQLite adapter
4. Retrieval endpoint
5. Store/observe endpoints
6. OpenClaw skill
7. Web adapter
8. Metrics/debug surface
9. Optional Contextual-assisted classification

## Future Steps

- Add namespace and user separation
- Add confidence and freshness scoring
- Add compaction and summarization for stale observations
- Add support for more tool families like REST APIs and internal CLIs
- Add lightweight UI or CLI inspection tools
- Add deployment of the Rust binary with Redis Cloud
