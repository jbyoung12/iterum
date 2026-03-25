from __future__ import annotations

from app.models import FactRecord, ObservationRecord, PlaybookRecord, stable_identifier


def make_search_then_fetch_playbook() -> PlaybookRecord:
    return PlaybookRecord(
        id="web_fetch_search_then_fetch",
        tool_name="web.fetch",
        error_family="invalid_url",
        title="Search before fetch when the input is not a URL",
        steps=[
            "If the provided value is not a valid URL, treat it as a search query.",
            "Search for the target page using the query text.",
            "Fetch the best matching result instead of retrying the invalid URL.",
        ],
        confidence=0.9,
    )


def make_preferred_domain_fact(topic: str, domain: str) -> FactRecord:
    return FactRecord(
        id=stable_identifier("fact", {"tool_name": "web.fetch", "topic": topic, "domain": domain}),
        tool_name="web.fetch",
        resource_id=None,
        topic="preferred_domain",
        title=f"Preferred domain for {topic}",
        content=f"Prefer {domain} for {topic} lookups when it is available.",
        confidence=0.8,
    )


def make_fetch_observation(resource_id: str, note: str) -> ObservationRecord:
    return ObservationRecord(
        id=stable_identifier("observation", {"tool_name": "web.fetch", "resource_id": resource_id, "note": note}),
        tool_name="web.fetch",
        resource_id=resource_id,
        topic="fetch_state",
        content=note,
        confidence=0.7,
        ttl_seconds=12 * 60 * 60,
    )
