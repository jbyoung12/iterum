from __future__ import annotations

from fastapi import APIRouter, Depends

from app.dependencies import get_store
from app.models import FactRecord, ObservationRecord, PlaybookRecord
from app.store import ContextStore

router = APIRouter(tags=["debug"])


@router.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok"}


@router.get("/v1/debug/facts", response_model=list[FactRecord])
def debug_facts(
    namespace: str,
    tool_name: str,
    resource_id: str | None = None,
    store: ContextStore = Depends(get_store),
) -> list[FactRecord]:
    return store.list_facts(namespace, tool_name, resource_id)


@router.get("/v1/debug/playbooks", response_model=list[PlaybookRecord])
def debug_playbooks(
    namespace: str,
    tool_name: str,
    error_family: str | None = None,
    store: ContextStore = Depends(get_store),
) -> list[PlaybookRecord]:
    return store.list_playbooks(namespace, tool_name, error_family)


@router.get("/v1/debug/observations", response_model=list[ObservationRecord])
def debug_observations(
    namespace: str,
    tool_name: str,
    resource_id: str | None = None,
    store: ContextStore = Depends(get_store),
) -> list[ObservationRecord]:
    return store.list_observations(namespace, tool_name, resource_id)
