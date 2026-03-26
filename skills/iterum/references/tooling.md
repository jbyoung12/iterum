# Iterum Tooling

## Current implementation (Rust / Axum)

- Context retrieval: `POST /v1/context/retrieve`
- Fact storage: `POST /v1/context/facts`
- Playbook storage: `POST /v1/context/playbooks`
- Observation storage: `POST /v1/context/observations`
- Tool-result event ingestion: `POST /v1/events/tool-result`
- Debug listing: `GET /v1/debug/facts`, `GET /v1/debug/playbooks`, `GET /v1/debug/observations`, `GET /v1/debug/constraints`, `GET /v1/debug/failure-patterns`
- Health check: `GET /health`
- Local helper scripts:
  - `scripts/retrieve_context.sh`
  - `scripts/store_fact.sh`
  - `scripts/store_observation.sh`

## Current operating model

- Facts store stable knowledge such as schemas, preferred domains, and tool argument shapes.
- Playbooks store reusable recovery procedures such as schema inspection before retrying a SQLite query.
- Observations store recent environment state such as an empty database or a broken endpoint.
- Constraints store negative knowledge learned from errors (e.g., "column X does not exist").
- Failure patterns track recurring errors with occurrence counts.

## Integration pattern

1. Before a brittle tool call, send `tool_name`, `resource_id`, and optional `error_text` to Iterum.
2. Inject the returned `prompt_context` into the agent prompt.
3. After resolving the task, write back stable facts or observations that will help the next run.
4. For repeated investigations, the second run should benefit from the first run's stored facts and observations.
