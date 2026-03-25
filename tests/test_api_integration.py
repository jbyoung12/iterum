from fastapi.testclient import TestClient

from app.main import app
from app.retrieval import ContextRetriever
from app.store import InMemoryContextStore


def test_store_and_retrieve_context_via_api() -> None:
    app.state.store = InMemoryContextStore()
    app.state.retriever = ContextRetriever(app.state.store, max_items=3)
    client = TestClient(app)

    fact_response = client.post(
        "/v1/context/facts",
        json={
            "namespace": "default",
            "tool_name": "sqlite3",
            "resource_id": "/tmp/demo.db",
            "topic": "schema",
            "title": "strategy_pnl schema",
            "content": "Columns: id, strategy, realized_pnl, unrealized_pnl, total_pnl, created_at",
            "confidence": 1.0,
        },
    )
    assert fact_response.status_code == 200

    playbook_response = client.post(
        "/v1/context/playbooks",
        json={
            "namespace": "default",
            "tool_name": "sqlite3",
            "error_family": "unknown_column",
            "title": "Inspect schema before retrying",
            "steps": [
                "Run .tables",
                "Run .schema <table>",
                "Rewrite after verifying columns",
            ],
            "confidence": 0.95,
        },
    )
    assert playbook_response.status_code == 200

    observation_response = client.post(
        "/v1/context/observations",
        json={
            "namespace": "default",
            "tool_name": "sqlite3",
            "resource_id": "/tmp/demo.db",
            "topic": "empty_state",
            "content": "Current snapshot: runs=0, fills=0, orders=0, positions=0",
            "confidence": 0.9,
        },
    )
    assert observation_response.status_code == 200

    retrieve_response = client.post(
        "/v1/context/retrieve",
        json={
            "namespace": "default",
            "tool_name": "sqlite3",
            "resource_id": "/tmp/demo.db",
            "task_type": "analysis",
            "error_text": "no such column: timestamp",
        },
    )
    assert retrieve_response.status_code == 200
    payload = retrieve_response.json()
    assert payload["matched_error_family"] == "unknown_column"
    assert len(payload["facts"]) >= 1
    assert len(payload["playbooks"]) >= 1
    assert len(payload["observations"]) >= 1
    assert "Relevant facts" in payload["prompt_context"]
