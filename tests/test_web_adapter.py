from app.adapters.web import make_fetch_observation, make_preferred_domain_fact, make_search_then_fetch_playbook


def test_search_then_fetch_playbook_helper() -> None:
    playbook = make_search_then_fetch_playbook()
    assert playbook.tool_name == "web.fetch"
    assert playbook.error_family == "invalid_url"
    assert any("Search" in step or "search" in step for step in playbook.steps)


def test_preferred_domain_fact_helper() -> None:
    fact = make_preferred_domain_fact("OpenClaw docs", "docs.openclaw.ai")
    assert fact.tool_name == "web.fetch"
    assert fact.topic == "preferred_domain"
    assert "docs.openclaw.ai" in fact.content


def test_fetch_observation_helper() -> None:
    observation = make_fetch_observation("https://docs.openclaw.ai", "Previous fetch returned a docs landing page.")
    assert observation.tool_name == "web.fetch"
    assert observation.resource_id == "https://docs.openclaw.ai"
    assert "landing page" in observation.content
