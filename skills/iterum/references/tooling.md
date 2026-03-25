# Iterum Tooling

## Current implementation

- Service endpoint: `POST /v1/kalshi/lookup`
- Memory inspection: `GET /v1/memory/{user_id}`
- Health check: `GET /health`

## Current demo repair

The scaffolded demo learns one deterministic repair:

- wrong key `ticker`
- fixed key `market_ticker`

Replace that rule in `app/kalshi_demo.py` once the real Kalshi example is provided.

## Demo requirements

- Show a first run that learns the fix
- Show a second run that reuses the fix from Redis
- Show the Redis-backed memory entry or the `/v1/memory/{user_id}` endpoint

