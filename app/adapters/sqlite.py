from __future__ import annotations

from app.models import FactRecord, ObservationRecord, PlaybookRecord, stable_identifier


def make_schema_fact(resource_id: str, table_name: str, columns: list[str], summary: str) -> FactRecord:
    return FactRecord(
        id=stable_identifier(
            "fact",
            {"tool_name": "sqlite3", "resource_id": resource_id, "topic": "schema", "table_name": table_name},
        ),
        tool_name="sqlite3",
        resource_id=resource_id,
        topic="schema",
        title=f"{table_name} schema",
        content=f"Columns: {', '.join(columns)}. {summary}",
        confidence=1.0,
    )


def make_empty_state_observation(resource_id: str, counts: dict[str, int]) -> ObservationRecord:
    ordered = ", ".join(f"{name}={count}" for name, count in sorted(counts.items()))
    return ObservationRecord(
        id=stable_identifier(
            "observation",
            {"tool_name": "sqlite3", "resource_id": resource_id, "topic": "empty_state", "counts": counts},
        ),
        tool_name="sqlite3",
        resource_id=resource_id,
        topic="empty_state",
        content=f"Current snapshot: {ordered}",
        confidence=0.9,
        ttl_seconds=24 * 60 * 60,
    )


def make_unknown_column_playbook() -> PlaybookRecord:
    return PlaybookRecord(
        id="sqlite_unknown_column_inspect_schema",
        tool_name="sqlite3",
        error_family="unknown_column",
        title="Inspect schema before retrying",
        steps=[
            "Run `.tables` to confirm the available tables.",
            "Run `.schema <table>` for the referenced table.",
            "Rewrite the query only after verifying the actual column names.",
        ],
        confidence=0.95,
    )
