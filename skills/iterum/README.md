# Iterum Skill for OpenClaw AI

Redis-backed operational context retrieval that helps agents avoid repeating the same mistakes.

## Quick Start

```bash
# 1. Start backend
cd /Users/joshuayoung/code/iterum
make run &

# 2. Install in OpenClaw
ln -s $(pwd)/skills/iterum ~/.openclaw/skills/iterum

# 3. Configure middleware (see OPENCLAW_INTEGRATION.md)

# 4. Use it
openclaw run "analyze btc_directional database"
# First run: learns schemas, makes errors
# Second run: uses stored context, no errors ✅
```

## Files

- `SKILL.md` - Skill definition and instructions
- `agents/openai.yaml` - OpenClaw agent configuration
- `tool_wrapper.py` - Automatic tool wrapping middleware
- `scripts/` - Helper scripts for context retrieval/storage
- `OPENCLAW_INTEGRATION.md` - Detailed integration guide

## How It Works

**Automatic mode:** Middleware intercepts brittle tool calls, retrieves context, injects it into prompts, and stores learnings.

**Manual mode:** Agents explicitly call `/iterum` before brittle operations.

See `OPENCLAW_INTEGRATION.md` for full details.
