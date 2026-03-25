from __future__ import annotations

from datetime import datetime

from app.models import ContextRetrieveRequest, FactRecord, ObservationRecord, PlaybookRecord


def _parse_iso(value: str) -> datetime:
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


def rank_facts(records: list[FactRecord], request: ContextRetrieveRequest, limit: int) -> list[FactRecord]:
    unique = {record.id: record for record in records}.values()
    ranked = sorted(
        unique,
        key=lambda record: (
            record.resource_id == request.resource_id,
            record.resource_id is None,
            record.confidence,
            _parse_iso(record.updated_at),
        ),
        reverse=True,
    )
    return ranked[:limit]


def rank_playbooks(records: list[PlaybookRecord], error_family: str | None, limit: int) -> list[PlaybookRecord]:
    unique = {record.id: record for record in records}.values()
    ranked = sorted(
        unique,
        key=lambda record: (
            record.error_family == error_family,
            record.error_family is None,
            record.confidence,
            _parse_iso(record.updated_at),
        ),
        reverse=True,
    )
    return ranked[:limit]


def rank_observations(records: list[ObservationRecord], request: ContextRetrieveRequest, limit: int) -> list[ObservationRecord]:
    unique = {record.id: record for record in records}.values()
    ranked = sorted(
        unique,
        key=lambda record: (
            record.resource_id == request.resource_id,
            record.resource_id is None,
            record.confidence,
            _parse_iso(record.updated_at),
        ),
        reverse=True,
    )
    return ranked[:limit]
