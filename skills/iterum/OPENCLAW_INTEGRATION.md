# OpenClaw AI Integration Guide

## Automatic Tool Wrapping (Option B/C)

This skill automatically wraps brittle tools (sqlite3, curl, etc.) to inject Iterum context before execution.

## Setup Steps

### 1. Start Iterum Backend

```bash
cd /Users/joshuayoung/code/iterum
source .venv/bin/activate
uvicorn app.main:app --host 127.0.0.1 --port 8000 &
```

Or use Docker:
```bash
make redis  # If using Redis
make run &
```

### 2. Install Skill in OpenClaw

Copy or symlink this skill directory to OpenClaw's skills directory:

```bash
# Find your OpenClaw installation
# Typical locations:
# - ~/.openclaw/skills/
# - ~/code/openclaw/skills/
# - /opt/openclaw/skills/

# Symlink the skill
ln -s /Users/joshuayoung/code/iterum/skills/iterum ~/.openclaw/skills/iterum

# Or copy it
cp -r /Users/joshuayoung/code/iterum/skills/iterum ~/.openclaw/skills/
```

### 3. Configure OpenClaw to Load the Middleware

In your OpenClaw agent configuration (usually `~/.openclaw/config.yaml` or similar):

```yaml
skills:
  iterum:
    enabled: true
    auto_inject: true  # Enable automatic tool wrapping

middleware:
  - name: iterum
    type: pre-tool
    config:
      base_url: http://127.0.0.1:8000
```

### 4. Verify Installation

```bash
# Check if Iterum is loaded
openclaw skills list | grep iterum

# Test retrieval
openclaw skill iterum retrieve sqlite3 /path/to/test.db
```

## How It Works

### First Run (Learning)
1. Agent starts task: "analyze btc_directional database"
2. **Middleware intercepts** sqlite3 tool call
3. Calls `retrieve_context.sh` → gets back "No context yet"
4. Executes query → gets error "no such column: timestamp"
5. Agent recovers by inspecting `.schema`
6. **Middleware stores** learned schema via `store_fact.sh`

### Second Run (Benefit)
1. Agent starts same task
2. **Middleware intercepts** sqlite3 tool call
3. Calls `retrieve_context.sh` → gets schema facts
4. **Injects context** into agent prompt:
   ```
   Known context:
   - strategy_pnl has columns: id, strategy, realized_pnl, unrealized_pnl, total_pnl, created_at
   - Database is currently empty (0 fills, 0 runs)
   ```
5. Agent uses correct columns immediately ✅
6. No more "no such column" errors ✅

## Manual Usage (Fallback)

If automatic wrapping doesn't work, agents can manually invoke:

```bash
# Before brittle tool use
/iterum retrieve sqlite3 /path/to/database.db

# After learning
/iterum store-fact sqlite3 /path/to/database.db schema "table_name schema" "columns: ..."
```

## Troubleshooting

### Middleware Not Activating

Check OpenClaw logs for middleware loading errors:
```bash
tail -f ~/.openclaw/logs/agent.log | grep iterum
```

### Backend Not Running

```bash
curl http://127.0.0.1:8000/health
# Should return: {"status":"ok"}
```

### Scripts Not Executable

```bash
chmod +x /Users/joshuayoung/code/iterum/skills/iterum/scripts/*.sh
```

## Architecture

```
┌─────────────┐
│ OpenClaw    │
│ Agent       │
└──────┬──────┘
       │
       │ 1. About to use sqlite3
       ▼
┌─────────────────┐
│ Iterum         │
│ Middleware     │◄─── tool_wrapper.py
└──────┬──────────┘
       │ 2. Retrieve context
       ▼
┌─────────────────┐
│ Iterum         │
│ Backend        │◄─── http://127.0.0.1:8000
│ (FastAPI)      │
└──────┬──────────┘
       │ 3. Query Redis/memory
       ▼
┌─────────────────┐
│ Redis/Memory   │◄─── Facts, Playbooks, Observations
└─────────────────┘
```

## Configuration Reference

### Environment Variables

- `ITERUM_BASE_URL` - Backend URL (default: http://127.0.0.1:8000)
- `ITERUM_REDIS_URL` - Redis URL (default: memory://local)
- `ITERUM_DEFAULT_NAMESPACE` - Default namespace (default: default)

### Skill Metadata

- `auto-inject-before` - Tools that trigger automatic context retrieval
- `middleware.enabled` - Enable/disable automatic wrapping
- `middleware.auto_store` - Automatically store learned facts

## Next Steps

1. ✅ Start Iterum backend
2. ✅ Install skill in OpenClaw
3. ⏳ Configure OpenClaw middleware
4. ⏳ Test with `/btc-directional` skill
5. ⏳ Verify second run uses stored context
