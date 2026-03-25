from __future__ import annotations

import hashlib
import json
from datetime import datetime, timezone
from typing import Any

from pydantic import BaseModel, Field


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def stable_identifier(prefix: str, payload: dict[str, Any]) -> str:
    digest = hashlib.sha256(json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")).hexdigest()
    return f"{prefix}_{digest[:12]}"


class FactRecord(BaseModel):
    id: str
    namespace: str = "default"
    tool_name: str
    resource_id: str | None = None
    topic: str
    title: str
    content: str
    confidence: float = 1.0
    updated_at: str = Field(default_factory=utc_now_iso)


class PlaybookRecord(BaseModel):
    id: str
    namespace: str = "default"
    tool_name: str
    error_family: str | None = None
    title: str
    steps: list[str] = Field(default_factory=list)
    confidence: float = 1.0
    updated_at: str = Field(default_factory=utc_now_iso)


class ObservationRecord(BaseModel):
    id: str
    namespace: str = "default"
    tool_name: str
    resource_id: str | None = None
    topic: str
    content: str
    confidence: float = 1.0
    updated_at: str = Field(default_factory=utc_now_iso)
    ttl_seconds: int | None = None


class ContextRetrieveRequest(BaseModel):
    namespace: str = "default"
    user_id: str | None = None
    tool_name: str
    resource_id: str | None = None
    task_type: str | None = None
    error_text: str | None = None
    query: str | None = None


class ContextRetrieveResponse(BaseModel):
    namespace: str
    tool_name: str
    resource_id: str | None = None
    matched_error_family: str | None = None
    facts: list[FactRecord] = Field(default_factory=list)
    playbooks: list[PlaybookRecord] = Field(default_factory=list)
    observations: list[ObservationRecord] = Field(default_factory=list)
    prompt_context: str


class StoreFactRequest(BaseModel):
    id: str | None = None
    namespace: str = "default"
    tool_name: str
    resource_id: str | None = None
    topic: str
    title: str
    content: str
    confidence: float = 1.0


class StorePlaybookRequest(BaseModel):
    id: str | None = None
    namespace: str = "default"
    tool_name: str
    error_family: str | None = None
    title: str
    steps: list[str] = Field(default_factory=list)
    confidence: float = 1.0


class StoreObservationRequest(BaseModel):
    id: str | None = None
    namespace: str = "default"
    tool_name: str
    resource_id: str | None = None
    topic: str
    content: str
    confidence: float = 1.0
    ttl_seconds: int | None = None
