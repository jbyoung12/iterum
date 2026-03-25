from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from app.mistake_memory import MistakeMemory
from app.models import LookupResponse


class KalshiToolError(Exception):
    pass


@dataclass
class DemoKalshiClient:
    tool_name: str

    def get_market(self, args: dict[str, Any]) -> dict[str, Any]:
        if "market_ticker" not in args:
            bad_keys = ", ".join(sorted(args.keys())) or "<none>"
            raise KalshiToolError(f"missing required field 'market_ticker'; got {bad_keys}")
        ticker = str(args["market_ticker"]).strip()
        if not ticker:
            raise KalshiToolError("missing required field 'market_ticker'; got empty value")
        return {
            "market_ticker": ticker,
            "status": "ok",
            "note": "Replace DemoKalshiClient with the real Kalshi integration once the example is fixed.",
        }


def repair_args(args: dict[str, Any]) -> tuple[dict[str, Any] | None, str | None]:
    if "ticker" in args and "market_ticker" not in args:
        fixed = dict(args)
        fixed["market_ticker"] = fixed.pop("ticker")
        return fixed, "mapped ticker -> market_ticker"
    if "event_ticker" in args and "market_ticker" not in args:
        fixed = dict(args)
        fixed["market_ticker"] = fixed.pop("event_ticker")
        return fixed, "mapped event_ticker -> market_ticker"
    return None, None


class SafeKalshiService:
    def __init__(self, tool_name: str, memory: MistakeMemory, client: DemoKalshiClient) -> None:
        self.tool_name = tool_name
        self.memory = memory
        self.client = client

    def lookup_market(self, user_id: str, args: dict[str, Any]) -> LookupResponse:
        remembered = self.memory.lookup(user_id, self.tool_name, args)
        if remembered is not None:
            self.memory.mark_hit(user_id, self.tool_name, args)
            result = self.client.get_market(remembered.fixed_args)
            return LookupResponse(
                ok=True,
                tool_name=self.tool_name,
                args_used=remembered.fixed_args,
                result=result,
                memory_hit=True,
            )

        try:
            result = self.client.get_market(args)
            return LookupResponse(
                ok=True,
                tool_name=self.tool_name,
                args_used=args,
                result=result,
            )
        except KalshiToolError as exc:
            fixed_args, _ = repair_args(args)
            if fixed_args is None:
                return LookupResponse(
                    ok=False,
                    tool_name=self.tool_name,
                    args_used=args,
                    error=str(exc),
                )
            result = self.client.get_market(fixed_args)
            self.memory.record(user_id, self.tool_name, args, fixed_args, str(exc))
            return LookupResponse(
                ok=True,
                tool_name=self.tool_name,
                args_used=fixed_args,
                result=result,
                learned_fix=True,
            )

