# Iterum

Iterum is a small Redis-backed mistake-memory service for OpenClaw demos. It learns one failing Kalshi tool-call shape, stores the fix in Redis, and applies that fix on later runs before the tool call fails again.

## Scope

- Redis is the durable memory layer
- FastAPI exposes the service endpoints
- A demo Kalshi wrapper shows the miss -> learn -> hit loop
- An OpenClaw skill tells agents when to use Iterum

## Run locally

```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -e ".[dev]"
docker compose up -d redis
uvicorn app.main:app --reload
```

## Demo flow

```bash
curl -s http://127.0.0.1:8000/v1/kalshi/lookup \
  -H 'content-type: application/json' \
  -d '{"user_id":"demo","args":{"ticker":"KXBTC-2026-03-25-YES"}}' | jq
```

Run the same command twice. The first request should learn the fix. The second should show a memory hit.

