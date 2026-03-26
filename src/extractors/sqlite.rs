use chrono::Utc;
use regex::Regex;

use crate::extractors::{Extracted, Extractor};
use crate::models::*;

pub struct SqliteExtractor;

impl Extractor for SqliteExtractor {
    fn can_handle(&self, event: &ToolResultEvent) -> bool {
        let cmd = event.command.as_deref().unwrap_or("");
        cmd.contains("sqlite3")
            || event
                .resource_id
                .as_deref()
                .is_some_and(|r| r.starts_with("sqlite:"))
    }

    fn extract(&self, event: &ToolResultEvent) -> Extracted {
        let mut result = Extracted::default();
        let cmd = event.command.as_deref().unwrap_or("");
        let stdout = event.stdout.as_deref().unwrap_or("");
        let stderr = event.stderr.as_deref().unwrap_or("");
        let text = if stdout.is_empty() { stderr } else { stdout };
        let resource_id = event.resource_id.clone();
        let now = Utc::now();

        // Detect .tables output
        if cmd.contains(".tables") && !text.is_empty() && !event.is_error {
            let id = stable_identifier(
                "fact",
                &serde_json::json!({
                    "namespace": event.namespace,
                    "tool_name": "sqlite3",
                    "resource_id": resource_id,
                    "topic": "tables",
                }),
            );
            result.facts.push(FactRecord {
                id,
                namespace: event.namespace.clone(),
                tool_name: "sqlite3".into(),
                resource_id: resource_id.clone(),
                topic: "tables".into(),
                title: format!(
                    "Known tables for {}",
                    resource_id.as_deref().unwrap_or("unknown")
                ),
                content: text.to_string(),
                confidence: 0.95,
                updated_at: now,
            });
        }

        // Detect .schema <table> or PRAGMA table_info(<table>)
        if let Some(table_name) = extract_schema_table(cmd) {
            if !text.is_empty() && !event.is_error {
                let id = stable_identifier(
                    "fact",
                    &serde_json::json!({
                        "namespace": event.namespace,
                        "tool_name": "sqlite3",
                        "resource_id": resource_id,
                        "topic": format!("schema:{table_name}"),
                    }),
                );
                result.facts.push(FactRecord {
                    id,
                    namespace: event.namespace.clone(),
                    tool_name: "sqlite3".into(),
                    resource_id: resource_id.clone(),
                    topic: format!("schema:{table_name}"),
                    title: format!("Schema for {table_name}"),
                    content: text.to_string(),
                    confidence: 0.95,
                    updated_at: now,
                });
            }
        }

        // Detect error patterns
        let combined = format!("{stdout}\n{stderr}");
        if let Some(error_family) = detect_sqlite_error(&combined) {
            let pattern_text = combined.lines().find(|l| {
                let low = l.to_lowercase();
                low.contains("error") || low.contains("no such")
            }).unwrap_or(&combined).trim().to_string();

            let fp_id = stable_identifier(
                "fp",
                &serde_json::json!({
                    "namespace": event.namespace,
                    "tool_name": "sqlite3",
                    "resource_id": resource_id,
                    "error_family": error_family,
                    "pattern": pattern_text,
                }),
            );
            result.failure_patterns.push(FailurePatternRecord {
                id: fp_id,
                namespace: event.namespace.clone(),
                tool_name: "sqlite3".into(),
                resource_id: resource_id.clone(),
                error_family: error_family.clone(),
                pattern: pattern_text.clone(),
                occurrence_count: 1,
                first_seen: now,
                last_seen: now,
                ttl_seconds: Some(86400),
            });

            // Also store as constraint for specific errors
            if error_family == "unknown_column" || error_family == "unknown_table" {
                let c_id = stable_identifier(
                    "constraint",
                    &serde_json::json!({
                        "namespace": event.namespace,
                        "tool_name": "sqlite3",
                        "resource_id": resource_id,
                        "error_family": error_family,
                        "pattern": pattern_text,
                    }),
                );
                result.constraints.push(ConstraintRecord {
                    id: c_id,
                    namespace: event.namespace.clone(),
                    tool_name: "sqlite3".into(),
                    resource_id: resource_id.clone(),
                    topic: error_family.clone(),
                    title: format!("Invalid reference: {error_family}"),
                    content: pattern_text,
                    confidence: 0.9,
                    updated_at: now,
                    ttl_seconds: Some(86400),
                });
            }
        }

        result
    }
}

fn extract_schema_table(command: &str) -> Option<String> {
    let schema_re = Regex::new(r"\.schema\s+([A-Za-z_][A-Za-z0-9_]*)").ok()?;
    if let Some(cap) = schema_re.captures(command) {
        return Some(cap[1].to_string());
    }
    let pragma_re =
        Regex::new(r"(?i)\bPRAGMA\s+table_info\s*\(\s*([A-Za-z_][A-Za-z0-9_]*)\s*\)").ok()?;
    if let Some(cap) = pragma_re.captures(command) {
        return Some(cap[1].to_string());
    }
    None
}

fn detect_sqlite_error(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    if lower.contains("no such column") || lower.contains("unknown column") {
        return Some("unknown_column".into());
    }
    if lower.contains("no such table") {
        return Some("unknown_table".into());
    }
    if lower.contains("syntax error") || lower.contains("near \"") {
        return Some("syntax_error".into());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_schema_table() {
        assert_eq!(
            extract_schema_table("sqlite3 db.db '.schema users'"),
            Some("users".into())
        );
        assert_eq!(
            extract_schema_table("sqlite3 db.db 'PRAGMA table_info(orders)'"),
            Some("orders".into())
        );
        assert_eq!(extract_schema_table("sqlite3 db.db '.tables'"), None);
    }

    #[test]
    fn test_can_handle() {
        let ext = SqliteExtractor;
        let event = ToolResultEvent {
            namespace: "default".into(),
            tool_name: "bash".into(),
            command: Some("sqlite3 ~/db.db '.tables'".into()),
            resource_id: None,
            stdout: None,
            stderr: None,
            is_error: false,
            agent_id: None,
            session_id: None,
        };
        assert!(ext.can_handle(&event));

        let event2 = ToolResultEvent {
            command: Some("ls -la".into()),
            resource_id: None,
            ..event
        };
        assert!(!ext.can_handle(&event2));
    }

    #[test]
    fn test_extract_tables() {
        let ext = SqliteExtractor;
        let event = ToolResultEvent {
            namespace: "default".into(),
            tool_name: "bash".into(),
            command: Some("sqlite3 ~/db.db '.tables'".into()),
            resource_id: Some("sqlite:~/db.db".into()),
            stdout: Some("users  orders  products".into()),
            stderr: None,
            is_error: false,
            agent_id: None,
            session_id: None,
        };
        let result = ext.extract(&event);
        assert_eq!(result.facts.len(), 1);
        assert!(result.facts[0].content.contains("users"));
    }

    #[test]
    fn test_extract_error() {
        let ext = SqliteExtractor;
        let event = ToolResultEvent {
            namespace: "default".into(),
            tool_name: "bash".into(),
            command: Some("sqlite3 ~/db.db 'SELECT bad FROM t'".into()),
            resource_id: Some("sqlite:~/db.db".into()),
            stdout: None,
            stderr: Some("Error: no such column: bad".into()),
            is_error: true,
            agent_id: None,
            session_id: None,
        };
        let result = ext.extract(&event);
        assert!(!result.failure_patterns.is_empty());
        assert!(!result.constraints.is_empty());
        assert_eq!(result.failure_patterns[0].error_family, "unknown_column");
    }
}
