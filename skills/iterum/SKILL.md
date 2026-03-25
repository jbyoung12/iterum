---
name: iterum
description: Redis-backed operational memory for OpenClaw agents that use Kalshi tools. Use when the agent is about to call a Kalshi lookup or trading tool and there is a risk of repeating a known input-shape mistake, enum mismatch, or ticker-format failure across runs.
---

# Iterum

Use Iterum before risky Kalshi tool calls that have failed before for the same user or namespace.

Prefer a safe wrapper flow over a direct tool call:

1. Send the intended Kalshi tool name and arguments to Iterum.
2. If Iterum returns a corrected argument shape, use the corrected version.
3. If the direct call still fails, record the failure and the fix so later runs avoid the same error.

## Workflow

Use this sequence:

1. Identify the concrete Kalshi tool being used, such as market lookup or order placement.
2. Check Iterum with the original arguments.
3. If Iterum returns a remembered fix, apply it before the Kalshi call.
4. If no fix exists, execute the Kalshi call normally.
5. If the call fails and the repair is deterministic, retry once with the repaired arguments and store the fix in Redis.
6. Surface whether the result came from a memory hit or a newly learned fix.

## Guardrails

- Auto-apply only deterministic fixes.
- Prefer single-field rewrites such as `ticker -> market_ticker` or enum normalization.
- Do not invent market tickers or trade parameters.
- If the fix is not obvious, stop and report the failure instead of guessing.

## Resources

Read [references/tooling.md](./references/tooling.md) when wiring the service into an OpenClaw demo or replacing the simulated Kalshi repair with the real example.
