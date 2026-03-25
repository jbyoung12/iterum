from app.adapters.sqlite import make_empty_state_observation, make_schema_fact, make_unknown_column_playbook


def test_schema_fact_helper() -> None:
    record = make_schema_fact("/tmp/demo.db", "strategy_pnl", ["id", "strategy", "created_at"], "No timestamp column.")
    assert record.tool_name == "sqlite3"
    assert record.resource_id == "/tmp/demo.db"
    assert record.topic == "schema"
    assert "strategy_pnl" in record.title
    assert "created_at" in record.content


def test_unknown_column_playbook_helper() -> None:
    playbook = make_unknown_column_playbook()
    assert playbook.tool_name == "sqlite3"
    assert playbook.error_family == "unknown_column"
    assert any(".schema" in step for step in playbook.steps)


def test_empty_state_observation_helper() -> None:
    observation = make_empty_state_observation("/tmp/demo.db", {"runs": 0, "fills": 0})
    assert observation.tool_name == "sqlite3"
    assert observation.resource_id == "/tmp/demo.db"
    assert "fills=0" in observation.content
