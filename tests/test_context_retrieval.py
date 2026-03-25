from app.models import ContextRetrieveRequest, FactRecord, ObservationRecord, PlaybookRecord
from app.retrieval import ContextRetriever
from app.store import InMemoryContextStore


def test_retrieval_prioritizes_exact_resource_matches() -> None:
    store = InMemoryContextStore()
    store.put_fact(
        FactRecord(
            id="fact_exact",
            namespace="default",
            tool_name="sqlite3",
            resource_id="/tmp/demo.db",
            topic="schema",
            title="Exact schema",
            content="Columns: id, created_at",
        )
    )
    store.put_fact(
        FactRecord(
            id="fact_generic",
            namespace="default",
            tool_name="sqlite3",
            resource_id=None,
            topic="schema",
            title="Generic sqlite fact",
            content="Use .schema before retrying.",
            confidence=0.8,
        )
    )
    store.put_playbook(
        PlaybookRecord(
            id="playbook_unknown_column",
            namespace="default",
            tool_name="sqlite3",
            error_family="unknown_column",
            title="Inspect schema first",
            steps=["Run .schema"],
        )
    )
    store.put_observation(
        ObservationRecord(
            id="obs_exact",
            namespace="default",
            tool_name="sqlite3",
            resource_id="/tmp/demo.db",
            topic="empty_state",
            content="runs=0, fills=0",
        )
    )

    retriever = ContextRetriever(store, max_items=3)
    response = retriever.retrieve(
        ContextRetrieveRequest(
            namespace="default",
            tool_name="sqlite3",
            resource_id="/tmp/demo.db",
            error_text="no such column: timestamp",
        )
    )

    assert response.matched_error_family == "unknown_column"
    assert response.facts[0].id == "fact_exact"
    assert response.playbooks[0].id == "playbook_unknown_column"
    assert response.observations[0].id == "obs_exact"
    assert "Relevant facts" in response.prompt_context
    assert "Recommended playbooks" in response.prompt_context
    assert "Recent observations" in response.prompt_context
