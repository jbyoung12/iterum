use chrono::Utc;

use crate::extractors::{Extracted, Extractor};
use crate::models::*;

pub struct CliExtractor;

impl Extractor for CliExtractor {
    fn can_handle(&self, event: &ToolResultEvent) -> bool {
        event.is_error
    }

    fn extract(&self, event: &ToolResultEvent) -> Extracted {
        let mut result = Extracted::default();
        let stderr = event.stderr.as_deref().unwrap_or("");
        let resource_id = event.resource_id.clone();
        let now = Utc::now();

        if let Some((error_family, pattern_text)) = detect_cli_error(stderr) {
            let fp_id = stable_identifier(
                "fp",
                &serde_json::json!({
                    "namespace": event.namespace,
                    "tool_name": event.tool_name,
                    "resource_id": resource_id,
                    "error_family": error_family,
                    "pattern": pattern_text,
                }),
            );
            result.failure_patterns.push(FailurePatternRecord {
                id: fp_id,
                namespace: event.namespace.clone(),
                tool_name: event.tool_name.clone(),
                resource_id: resource_id.clone(),
                error_family: error_family.clone(),
                pattern: pattern_text.clone(),
                occurrence_count: 1,
                first_seen: now,
                last_seen: now,
                ttl_seconds: Some(86400),
            });

            if error_family == "unknown_flag" || error_family == "missing_arg" {
                let c_id = stable_identifier(
                    "constraint",
                    &serde_json::json!({
                        "namespace": event.namespace,
                        "tool_name": event.tool_name,
                        "resource_id": resource_id,
                        "error_family": error_family,
                        "pattern": pattern_text,
                    }),
                );
                result.constraints.push(ConstraintRecord {
                    id: c_id,
                    namespace: event.namespace.clone(),
                    tool_name: event.tool_name.clone(),
                    resource_id,
                    topic: error_family.clone(),
                    title: format!("CLI error: {error_family}"),
                    content: pattern_text,
                    confidence: 0.8,
                    updated_at: now,
                    ttl_seconds: Some(86400),
                });
            }
        }

        result
    }
}

fn detect_cli_error(text: &str) -> Option<(String, String)> {
    let lower = text.to_lowercase();
    let pattern_line = text
        .lines()
        .find(|l| {
            let low = l.to_lowercase();
            low.contains("unknown")
                || low.contains("unrecognized")
                || low.contains("invalid")
                || low.contains("missing")
                || low.contains("error")
        })
        .unwrap_or(text)
        .trim()
        .to_string();

    if lower.contains("unknown option")
        || lower.contains("unknown flag")
        || lower.contains("unrecognized option")
        || lower.contains("invalid option")
    {
        return Some(("unknown_flag".into(), pattern_line));
    }
    if lower.contains("missing argument")
        || lower.contains("requires a value")
        || lower.contains("missing required")
    {
        return Some(("missing_arg".into(), pattern_line));
    }
    None
}
