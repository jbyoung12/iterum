from __future__ import annotations

import hashlib
import json
import re
from datetime import datetime, timezone
from typing import Any

from app.models import MemoryEntry
from app.store import MistakeStore


def canonical_json(data: dict[str, Any]) -> str:
    return json.dumps(data, sort_keys=True, separators=(",", ":"))


def args_signature(data: dict[str, Any]) -> str:
    return hashlib.sha256(canonical_json(data).encode("utf-8")).hexdigest()[:16]


def normalize_error(message: str) -> str:
    normalized = message.strip().lower()
    normalized = re.sub(r"\s+", " ", normalized)
    normalized = re.sub(r"'[^']+'", "'<value>'", normalized)
    normalized = re.sub(r"\b[a-z0-9]{8,}\b", "<token>", normalized)
    return normalized


class MistakeMemory:
    def __init__(self, store: MistakeStore) -> None:
        self.store = store

    def lookup(self, user_id: str, tool_name: str, bad_args: dict[str, Any]) -> MemoryEntry | None:
        return self.store.get(user_id, tool_name, args_signature(bad_args))

    def record(
        self,
        user_id: str,
        tool_name: str,
        bad_args: dict[str, Any],
        fixed_args: dict[str, Any],
        error_message: str,
    ) -> MemoryEntry:
        now = datetime.now(timezone.utc).isoformat()
        signature = args_signature(bad_args)
        existing = self.store.get(user_id, tool_name, signature)
        if existing is None:
            entry = MemoryEntry(
                user_id=user_id,
                tool_name=tool_name,
                args_signature=signature,
                bad_args=bad_args,
                fixed_args=fixed_args,
                error_signature=normalize_error(error_message),
                success_count=1,
                hit_count=0,
                created_at=now,
                updated_at=now,
            )
        else:
            entry = existing
            entry.fixed_args = fixed_args
            entry.error_signature = normalize_error(error_message)
            entry.success_count += 1
            entry.updated_at = now
        self.store.put(entry)
        return entry

    def mark_hit(self, user_id: str, tool_name: str, bad_args: dict[str, Any]) -> MemoryEntry | None:
        return self.store.increment_hit(user_id, tool_name, args_signature(bad_args))

