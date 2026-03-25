from __future__ import annotations

import json
from datetime import datetime, timezone
from typing import Protocol

from redis import Redis

from app.models import MemoryEntry


class MistakeStore(Protocol):
    def get(self, user_id: str, tool_name: str, args_signature: str) -> MemoryEntry | None:
        ...

    def put(self, entry: MemoryEntry) -> None:
        ...

    def increment_hit(self, user_id: str, tool_name: str, args_signature: str) -> MemoryEntry | None:
        ...

    def list_entries(self, user_id: str, tool_name: str | None = None) -> list[MemoryEntry]:
        ...


class RedisMistakeStore:
    def __init__(self, client: Redis, ttl_seconds: int) -> None:
        self.client = client
        self.ttl_seconds = ttl_seconds

    @staticmethod
    def _key(user_id: str, tool_name: str, args_signature: str) -> str:
        return f"mistake:{user_id}:{tool_name}:{args_signature}"

    def get(self, user_id: str, tool_name: str, args_signature: str) -> MemoryEntry | None:
        payload = self.client.get(self._key(user_id, tool_name, args_signature))
        if not payload:
            return None
        data = json.loads(payload)
        return MemoryEntry.model_validate(data)

    def put(self, entry: MemoryEntry) -> None:
        key = self._key(entry.user_id, entry.tool_name, entry.args_signature)
        self.client.set(key, entry.model_dump_json(), ex=self.ttl_seconds)

    def increment_hit(self, user_id: str, tool_name: str, args_signature: str) -> MemoryEntry | None:
        entry = self.get(user_id, tool_name, args_signature)
        if entry is None:
            return None
        entry.hit_count += 1
        entry.updated_at = datetime.now(timezone.utc).isoformat()
        self.put(entry)
        return entry

    def list_entries(self, user_id: str, tool_name: str | None = None) -> list[MemoryEntry]:
        pattern = f"mistake:{user_id}:{tool_name or '*'}:*"
        keys = sorted(self.client.scan_iter(match=pattern))
        entries: list[MemoryEntry] = []
        for key in keys:
            payload = self.client.get(key)
            if payload:
                entries.append(MemoryEntry.model_validate(json.loads(payload)))
        return entries


class InMemoryMistakeStore:
    def __init__(self) -> None:
        self.data: dict[tuple[str, str, str], MemoryEntry] = {}

    def get(self, user_id: str, tool_name: str, args_signature: str) -> MemoryEntry | None:
        return self.data.get((user_id, tool_name, args_signature))

    def put(self, entry: MemoryEntry) -> None:
        self.data[(entry.user_id, entry.tool_name, entry.args_signature)] = entry

    def increment_hit(self, user_id: str, tool_name: str, args_signature: str) -> MemoryEntry | None:
        entry = self.get(user_id, tool_name, args_signature)
        if entry is None:
            return None
        entry.hit_count += 1
        self.put(entry)
        return entry

    def list_entries(self, user_id: str, tool_name: str | None = None) -> list[MemoryEntry]:
        return [
            entry
            for entry in self.data.values()
            if entry.user_id == user_id and (tool_name is None or entry.tool_name == tool_name)
        ]
