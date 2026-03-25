# Iterum Tooling

## Current implementation

- Context retrieval: `POST /v1/context/retrieve`
- Fact storage: `POST /v1/context/facts`
- Playbook storage: `POST /v1/context/playbooks`
- Observation storage: `POST /v1/context/observations`
- Debug listing: `GET /v1/debug/facts`, `GET /v1/debug/playbooks`, `GET /v1/debug/observations`
- Health check: `GET /health`

## Current operating model

- Facts store stable knowledge such as schemas, preferred domains, and tool argument shapes.
- Playbooks store reusable recovery procedures such as schema inspection before retrying a SQLite query.
- Observations store recent environment state such as an empty database or a broken endpoint.

## Integration pattern

1. Before a brittle tool call, send `tool_name`, `resource_id`, and optional `error_text` to Iterum.
2. Inject the returned `prompt_context` into the agent prompt.
3. After resolving the task, write back stable facts or observations that will help the next run.
