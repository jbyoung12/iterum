from __future__ import annotations

from fastapi import Depends, FastAPI
from redis import Redis

from app.config import Settings, get_settings
from app.kalshi_demo import DemoKalshiClient, SafeKalshiService
from app.mistake_memory import MistakeMemory
from app.models import LookupRequest, LookupResponse, ManualRecordRequest
from app.store import RedisMistakeStore


def build_services(settings: Settings) -> tuple[MistakeMemory, SafeKalshiService]:
    redis_client = Redis.from_url(settings.redis_url, decode_responses=True)
    store = RedisMistakeStore(redis_client, ttl_seconds=settings.memory_ttl_seconds)
    memory = MistakeMemory(store)
    kalshi = SafeKalshiService(
        tool_name=settings.kalshi_tool_name,
        memory=memory,
        client=DemoKalshiClient(tool_name=settings.kalshi_tool_name),
    )
    return memory, kalshi


app = FastAPI(title="Iterum")


def get_memory(settings: Settings = Depends(get_settings)) -> MistakeMemory:
    memory, _ = build_services(settings)
    return memory


def get_kalshi(settings: Settings = Depends(get_settings)) -> SafeKalshiService:
    _, kalshi = build_services(settings)
    return kalshi


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok"}


@app.post("/v1/kalshi/lookup", response_model=LookupResponse)
def lookup_market(
    request: LookupRequest,
    kalshi: SafeKalshiService = Depends(get_kalshi),
) -> LookupResponse:
    return kalshi.lookup_market(request.user_id, request.args)


@app.get("/v1/memory/{user_id}")
def list_memory(
    user_id: str,
    tool_name: str | None = None,
    memory: MistakeMemory = Depends(get_memory),
) -> list[dict]:
    return [entry.model_dump() for entry in memory.store.list_entries(user_id, tool_name)]


@app.post("/v1/memory/record")
def manual_record(
    request: ManualRecordRequest,
    memory: MistakeMemory = Depends(get_memory),
) -> dict:
    entry = memory.record(
        user_id=request.user_id,
        tool_name=request.tool_name,
        bad_args=request.bad_args,
        fixed_args=request.fixed_args,
        error_message=request.error_signature,
    )
    return entry.model_dump()

