use crate::models::*;

/// Build a compact 2-6 bullet hint string ordered by importance:
/// 1. Facts (schemas, valid columns)
/// 2. Constraints (invalid columns, bad flags)
/// 3. Failure patterns (only recurring, with count)
/// Falls back to old-style sections for playbooks/observations.
pub fn format_prompt_context(
    facts: &[FactRecord],
    playbooks: &[PlaybookRecord],
    observations: &[ObservationRecord],
    constraints: &[ConstraintRecord],
    failure_patterns: &[FailurePatternRecord],
) -> String {
    let mut bullets: Vec<String> = Vec::new();

    // 1. Facts first
    for fact in facts {
        bullets.push(format!("{}: {}", fact.title, fact.content));
    }

    // 2. Constraints second
    for constraint in constraints {
        bullets.push(format!("[constraint] {}: {}", constraint.title, constraint.content));
    }

    // 3. Failure patterns last (with count)
    for fp in failure_patterns {
        bullets.push(format!(
            "[failure x{}] {}: {}",
            fp.occurrence_count, fp.error_family, fp.pattern
        ));
    }

    // 4. Playbooks
    for playbook in playbooks {
        let steps_str = playbook.steps.join("; ");
        bullets.push(format!("{}: {}", playbook.title, steps_str));
    }

    // 5. Observations
    for obs in observations {
        bullets.push(obs.content.clone());
    }

    if bullets.is_empty() {
        return "No relevant stored context.".into();
    }

    // Cap at 6 bullets for compact output
    bullets.truncate(6);
    bullets
        .iter()
        .map(|b| format!("- {b}"))
        .collect::<Vec<_>>()
        .join("\n")
}
