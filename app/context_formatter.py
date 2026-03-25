from __future__ import annotations

from app.models import FactRecord, ObservationRecord, PlaybookRecord


def _section(title: str, lines: list[str]) -> str:
    if not lines:
        return ""
    body = "\n".join(f"- {line}" for line in lines)
    return f"{title}:\n{body}"


def format_prompt_context(
    facts: list[FactRecord],
    playbooks: list[PlaybookRecord],
    observations: list[ObservationRecord],
) -> str:
    sections = [
        _section("Relevant facts", [f"{fact.title}: {fact.content}" for fact in facts]),
        _section(
            "Recommended playbooks",
            [f"{playbook.title}: {'; '.join(playbook.steps)}" for playbook in playbooks],
        ),
        _section("Recent observations", [observation.content for observation in observations]),
    ]
    text = "\n\n".join(section for section in sections if section)
    return text or "No relevant stored context."
