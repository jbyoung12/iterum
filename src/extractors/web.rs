use chrono::Utc;

use crate::extractors::{Extracted, Extractor};
use crate::models::*;

pub struct WebExtractor;

impl Extractor for WebExtractor {
    fn can_handle(&self, event: &ToolResultEvent) -> bool {
        let cmd = event.command.as_deref().unwrap_or("");
        cmd.contains("curl") || cmd.contains("wget") || cmd.contains("http")
    }

    fn extract(&self, event: &ToolResultEvent) -> Extracted {
        let mut result = Extracted::default();
        let stderr = event.stderr.as_deref().unwrap_or("");
        let stdout = event.stdout.as_deref().unwrap_or("");
        let combined = format!("{stdout}\n{stderr}");
        let resource_id = event.resource_id.clone();
        let now = Utc::now();

        if let Some(error_family) = detect_web_error(&combined) {
            let pattern_text = combined
                .lines()
                .find(|l| {
                    let low = l.to_lowercase();
                    low.contains("404")
                        || low.contains("not found")
                        || low.contains("invalid")
                        || low.contains("error")
                })
                .unwrap_or("HTTP error")
                .trim()
                .to_string();

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

            if error_family == "invalid_url" {
                let c_id = stable_identifier(
                    "constraint",
                    &serde_json::json!({
                        "namespace": event.namespace,
                        "tool_name": event.tool_name,
                        "resource_id": resource_id,
                        "pattern": pattern_text,
                    }),
                );
                result.constraints.push(ConstraintRecord {
                    id: c_id,
                    namespace: event.namespace.clone(),
                    tool_name: event.tool_name.clone(),
                    resource_id,
                    topic: "invalid_url".into(),
                    title: "Invalid URL detected".into(),
                    content: pattern_text,
                    confidence: 0.85,
                    updated_at: now,
                    ttl_seconds: Some(86400),
                });
            }
        }

        result
    }
}

fn detect_web_error(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    if lower.contains("404") || lower.contains("not found") {
        return Some("not_found".into());
    }
    if lower.contains("invalid url") || lower.contains("not a url") || lower.contains("malformed")
    {
        return Some("invalid_url".into());
    }
    None
}
