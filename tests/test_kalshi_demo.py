from app.kalshi_demo import DemoKalshiClient, SafeKalshiService
from app.mistake_memory import MistakeMemory
from app.store import InMemoryMistakeStore


def test_learns_fix_then_reuses_it() -> None:
    memory = MistakeMemory(InMemoryMistakeStore())
    service = SafeKalshiService(
        tool_name="kalshi.get_market",
        memory=memory,
        client=DemoKalshiClient(tool_name="kalshi.get_market"),
    )

    first = service.lookup_market("demo", {"ticker": "KXBTC-2026-03-25-YES"})
    assert first.ok is True
    assert first.learned_fix is True
    assert first.memory_hit is False
    assert first.args_used == {"market_ticker": "KXBTC-2026-03-25-YES"}

    second = service.lookup_market("demo", {"ticker": "KXBTC-2026-03-25-YES"})
    assert second.ok is True
    assert second.learned_fix is False
    assert second.memory_hit is True
    assert second.args_used == {"market_ticker": "KXBTC-2026-03-25-YES"}

