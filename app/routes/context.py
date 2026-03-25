from __future__ import annotations

from fastapi import APIRouter, Depends

from app.dependencies import get_retriever, get_store
from app.models import (
    ContextRetrieveRequest,
    ContextRetrieveResponse,
    FactRecord,
    ObservationRecord,
    PlaybookRecord,
    StoreFactRequest,
    StoreObservationRequest,
    StorePlaybookRequest,
    stable_identifier,
)
from app.retrieval import ContextRetriever
from app.store import ContextStore

router = APIRouter(prefix="/v1/context", tags=["context"])


def _fact_from_request(request: StoreFactRequest) -> FactRecord:
    payload = request.model_dump(exclude={"id"})
    return FactRecord(id=request.id or stable_identifier("fact", payload), **payload)


def _playbook_from_request(request: StorePlaybookRequest) -> PlaybookRecord:
    payload = request.model_dump(exclude={"id"})
    return PlaybookRecord(id=request.id or stable_identifier("playbook", payload), **payload)


def _observation_from_request(request: StoreObservationRequest) -> ObservationRecord:
    payload = request.model_dump(exclude={"id"})
    return ObservationRecord(id=request.id or stable_identifier("observation", payload), **payload)


@router.post("/retrieve", response_model=ContextRetrieveResponse)
def retrieve_context(
    request: ContextRetrieveRequest,
    retriever: ContextRetriever = Depends(get_retriever),
) -> ContextRetrieveResponse:
    return retriever.retrieve(request)


@router.post("/facts", response_model=FactRecord)
def store_fact(
    request: StoreFactRequest,
    store: ContextStore = Depends(get_store),
) -> FactRecord:
    return store.put_fact(_fact_from_request(request))


@router.post("/playbooks", response_model=PlaybookRecord)
def store_playbook(
    request: StorePlaybookRequest,
    store: ContextStore = Depends(get_store),
) -> PlaybookRecord:
    return store.put_playbook(_playbook_from_request(request))


@router.post("/observations", response_model=ObservationRecord)
def store_observation(
    request: StoreObservationRequest,
    store: ContextStore = Depends(get_store),
) -> ObservationRecord:
    return store.put_observation(_observation_from_request(request))
