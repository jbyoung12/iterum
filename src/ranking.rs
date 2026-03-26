use crate::models::*;

pub fn rank_facts(
    records: Vec<FactRecord>,
    request: &ContextRetrieveRequest,
    limit: usize,
) -> Vec<FactRecord> {
    let mut deduped: Vec<FactRecord> = {
        let mut seen = std::collections::HashMap::new();
        for r in records {
            seen.entry(r.id.clone()).or_insert(r);
        }
        seen.into_values().collect()
    };
    deduped.sort_by(|a, b| {
        let score = |r: &FactRecord| -> (bool, bool, u64, i64) {
            (
                r.resource_id.as_deref() == request.resource_id.as_deref(),
                r.resource_id.is_none(),
                (r.confidence * 1000.0) as u64,
                r.updated_at.timestamp(),
            )
        };
        score(b).cmp(&score(a))
    });
    deduped.truncate(limit);
    deduped
}

pub fn rank_playbooks(
    records: Vec<PlaybookRecord>,
    error_family: Option<&str>,
    limit: usize,
) -> Vec<PlaybookRecord> {
    let mut deduped: Vec<PlaybookRecord> = {
        let mut seen = std::collections::HashMap::new();
        for r in records {
            seen.entry(r.id.clone()).or_insert(r);
        }
        seen.into_values().collect()
    };
    deduped.sort_by(|a, b| {
        let score = |r: &PlaybookRecord| -> (bool, bool, u64, i64) {
            (
                r.error_family.as_deref() == error_family,
                r.error_family.is_none(),
                (r.confidence * 1000.0) as u64,
                r.updated_at.timestamp(),
            )
        };
        score(b).cmp(&score(a))
    });
    deduped.truncate(limit);
    deduped
}

pub fn rank_observations(
    records: Vec<ObservationRecord>,
    request: &ContextRetrieveRequest,
    limit: usize,
) -> Vec<ObservationRecord> {
    let mut deduped: Vec<ObservationRecord> = {
        let mut seen = std::collections::HashMap::new();
        for r in records {
            seen.entry(r.id.clone()).or_insert(r);
        }
        seen.into_values().collect()
    };
    deduped.sort_by(|a, b| {
        let score = |r: &ObservationRecord| -> (bool, bool, u64, i64) {
            (
                r.resource_id.as_deref() == request.resource_id.as_deref(),
                r.resource_id.is_none(),
                (r.confidence * 1000.0) as u64,
                r.updated_at.timestamp(),
            )
        };
        score(b).cmp(&score(a))
    });
    deduped.truncate(limit);
    deduped
}

pub fn rank_constraints(
    records: Vec<ConstraintRecord>,
    request: &ContextRetrieveRequest,
    limit: usize,
) -> Vec<ConstraintRecord> {
    let mut deduped: Vec<ConstraintRecord> = {
        let mut seen = std::collections::HashMap::new();
        for r in records {
            seen.entry(r.id.clone()).or_insert(r);
        }
        seen.into_values().collect()
    };
    deduped.sort_by(|a, b| {
        let score = |r: &ConstraintRecord| -> (bool, bool, u64, i64) {
            (
                r.resource_id.as_deref() == request.resource_id.as_deref(),
                r.resource_id.is_none(),
                (r.confidence * 1000.0) as u64,
                r.updated_at.timestamp(),
            )
        };
        score(b).cmp(&score(a))
    });
    deduped.truncate(limit);
    deduped
}

pub fn rank_failure_patterns(
    records: Vec<FailurePatternRecord>,
    request: &ContextRetrieveRequest,
    limit: usize,
) -> Vec<FailurePatternRecord> {
    let mut deduped: Vec<FailurePatternRecord> = {
        let mut seen = std::collections::HashMap::new();
        for r in records {
            seen.entry(r.id.clone()).or_insert(r);
        }
        seen.into_values().collect()
    };
    // Only show recurring patterns (count > 1)
    deduped.retain(|r| r.occurrence_count > 1);
    deduped.sort_by(|a, b| {
        let score = |r: &FailurePatternRecord| -> (bool, bool, u32, i64) {
            (
                r.resource_id.as_deref() == request.resource_id.as_deref(),
                r.resource_id.is_none(),
                r.occurrence_count,
                r.last_seen.timestamp(),
            )
        };
        score(b).cmp(&score(a))
    });
    deduped.truncate(limit);
    deduped
}
