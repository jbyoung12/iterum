from __future__ import annotations

from datetime import datetime, timezone
from typing import Any

from pydantic import BaseModel, Field


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


class MemoryEntry(BaseModel):
    user_id: str
    tool_name: str
    args_signature: str
    bad_args: dict[str, Any]
    fixed_args: dict[str, Any]
    error_signature: str
    success_count: int = 0
    hit_count: int = 0
    created_at: str = Field(default_factory=utc_now_iso)
    updated_at: str = Field(default_factory=utc_now_iso)


class LookupRequest(BaseModel):
    user_id: str = "demo"
    args: dict[str, Any]


class LookupResponse(BaseModel):
    ok: bool
    tool_name: str
    args_used: dict[str, Any]
    result: dict[str, Any] | None = None
    memory_hit: bool = False
    learned_fix: bool = False
    error: str | None = None


class ManualRecordRequest(BaseModel):
    user_id: str
    tool_name: str
    bad_args: dict[str, Any]
    fixed_args: dict[str, Any]
    error_signature: str

