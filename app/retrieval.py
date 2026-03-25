from __future__ import annotations

from app.context_formatter import format_prompt_context
from app.models import ContextRetrieveRequest, ContextRetrieveResponse
from app.ranking import rank_facts, rank_observations, rank_playbooks
from app.store import ContextStore


def infer_error_family(error_text: str | None) -> str | None:
    if not error_text:
        return None
    lowered = error_text.lower()
    if "no such column" in lowered or "unknown column" in lowered:
        return "unknown_column"
    if "no such table" in lowered:
        return "unknown_table"
    if "not found" in lowered or "404" in lowered:
        return "not_found"
    if "invalid url" in lowered or "not a url" in lowered:
        return "invalid_url"
    if "missing required" in lowered:
        return "missing_required_field"
    return None


class ContextRetriever:
    def __init__(self, store: ContextStore, max_items: int = 3) -> None:
        self.store = store
        self.max_items = max_items

    def retrieve(self, request: ContextRetrieveRequest) -> ContextRetrieveResponse:
        error_family = infer_error_family(request.error_text)
        facts = self.store.list_facts(request.namespace, request.tool_name, request.resource_id)
        playbooks = self.store.list_playbooks(request.namespace, request.tool_name, error_family)
        observations = self.store.list_observations(request.namespace, request.tool_name, request.resource_id)

        ranked_facts = rank_facts(facts, request, self.max_items)
        ranked_playbooks = rank_playbooks(playbooks, error_family, self.max_items)
        ranked_observations = rank_observations(observations, request, self.max_items)

        return ContextRetrieveResponse(
            namespace=request.namespace,
            tool_name=request.tool_name,
            resource_id=request.resource_id,
            matched_error_family=error_family,
            facts=ranked_facts,
            playbooks=ranked_playbooks,
            observations=ranked_observations,
            prompt_context=format_prompt_context(ranked_facts, ranked_playbooks, ranked_observations),
        )
