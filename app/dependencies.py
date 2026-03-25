from __future__ import annotations

from functools import lru_cache

from fastapi import Request
from redis import Redis

from app.config import Settings, get_settings
from app.retrieval import ContextRetriever
from app.store import ContextStore, InMemoryContextStore, RedisContextStore


@lru_cache(maxsize=1)
def get_redis_client(redis_url: str) -> Redis:
    return Redis.from_url(redis_url, decode_responses=True)


@lru_cache(maxsize=1)
def get_memory_store() -> InMemoryContextStore:
    return InMemoryContextStore()


def create_store(settings: Settings) -> ContextStore:
    if settings.redis_url.startswith("memory://"):
        return get_memory_store()
    return RedisContextStore(get_redis_client(settings.redis_url), ttl_seconds=settings.memory_ttl_seconds)


def create_retriever(store: ContextStore, settings: Settings) -> ContextRetriever:
    return ContextRetriever(store, max_items=settings.max_context_items)


def get_store(request: Request) -> ContextStore:
    return getattr(request.app.state, "store", create_store(get_settings()))


def get_retriever(request: Request) -> ContextRetriever:
    settings = get_settings()
    return getattr(request.app.state, "retriever", create_retriever(get_store(request), settings))
