use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

pub fn stable_identifier(prefix: &str, payload: &serde_json::Value) -> String {
    let canonical = serde_json::to_string(payload).unwrap_or_default();
    let hash = Sha256::digest(canonical.as_bytes());
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    format!("{prefix}_{}", &hex[..12])
}

// ---------------------------------------------------------------------------
// Record types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactRecord {
    pub id: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    pub topic: String,
    pub title: String,
    pub content: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default = "utc_now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookRecord {
    pub id: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_family: Option<String>,
    pub title: String,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default = "utc_now")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationRecord {
    pub id: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    pub topic: String,
    pub content: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default = "utc_now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintRecord {
    pub id: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    pub topic: String,
    pub title: String,
    pub content: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default = "utc_now")]
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePatternRecord {
    pub id: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub tool_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    pub error_family: String,
    pub pattern: String,
    #[serde(default = "default_one")]
    pub occurrence_count: u32,
    #[serde(default = "utc_now")]
    pub first_seen: DateTime<Utc>,
    #[serde(default = "utc_now")]
    pub last_seen: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ttl_seconds: Option<u64>,
}

// ---------------------------------------------------------------------------
// Request / Response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ContextRetrieveRequest {
    #[serde(default = "default_namespace")]
    pub namespace: String,
    #[serde(default)]
    pub user_id: Option<String>,
    pub tool_name: String,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub task_type: Option<String>,
    #[serde(default)]
    pub error_text: Option<String>,
    #[serde(default)]
    pub query: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ContextRetrieveResponse {
    pub namespace: String,
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_error_family: Option<String>,
    pub facts: Vec<FactRecord>,
    pub playbooks: Vec<PlaybookRecord>,
    pub observations: Vec<ObservationRecord>,
    pub constraints: Vec<ConstraintRecord>,
    pub failure_patterns: Vec<FailurePatternRecord>,
    pub prompt_context: String,
}

#[derive(Debug, Deserialize)]
pub struct StoreFactRequest {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub tool_name: String,
    #[serde(default)]
    pub resource_id: Option<String>,
    pub topic: String,
    pub title: String,
    pub content: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct StorePlaybookRequest {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub tool_name: String,
    #[serde(default)]
    pub error_family: Option<String>,
    pub title: String,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

#[derive(Debug, Deserialize)]
pub struct StoreObservationRequest {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub tool_name: String,
    #[serde(default)]
    pub resource_id: Option<String>,
    pub topic: String,
    pub content: String,
    #[serde(default = "default_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ToolResultEvent {
    #[serde(default = "default_namespace")]
    pub namespace: String,
    pub tool_name: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub stdout: Option<String>,
    #[serde(default)]
    pub stderr: Option<String>,
    #[serde(default)]
    pub is_error: bool,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ToolResultResponse {
    pub stored: Vec<StoredItem>,
}

#[derive(Debug, Serialize)]
pub struct StoredItem {
    pub record_type: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct DebugQuery {
    pub namespace: String,
    pub tool_name: String,
    #[serde(default)]
    pub resource_id: Option<String>,
    #[serde(default)]
    pub error_family: Option<String>,
}

// ---------------------------------------------------------------------------
// Defaults
// ---------------------------------------------------------------------------

fn default_namespace() -> String {
    "default".into()
}

fn default_confidence() -> f64 {
    1.0
}

fn default_one() -> u32 {
    1
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

impl StoreFactRequest {
    pub fn into_record(self) -> FactRecord {
        let id = self.id.unwrap_or_else(|| {
            let payload = serde_json::json!({
                "namespace": self.namespace,
                "tool_name": self.tool_name,
                "resource_id": self.resource_id,
                "topic": self.topic,
                "title": self.title,
                "content": self.content,
                "confidence": self.confidence,
            });
            stable_identifier("fact", &payload)
        });
        FactRecord {
            id,
            namespace: self.namespace,
            tool_name: self.tool_name,
            resource_id: self.resource_id,
            topic: self.topic,
            title: self.title,
            content: self.content,
            confidence: self.confidence,
            updated_at: Utc::now(),
        }
    }
}

impl StorePlaybookRequest {
    pub fn into_record(self) -> PlaybookRecord {
        let id = self.id.unwrap_or_else(|| {
            let payload = serde_json::json!({
                "namespace": self.namespace,
                "tool_name": self.tool_name,
                "error_family": self.error_family,
                "title": self.title,
                "steps": self.steps,
                "confidence": self.confidence,
            });
            stable_identifier("playbook", &payload)
        });
        PlaybookRecord {
            id,
            namespace: self.namespace,
            tool_name: self.tool_name,
            error_family: self.error_family,
            title: self.title,
            steps: self.steps,
            confidence: self.confidence,
            updated_at: Utc::now(),
        }
    }
}

impl StoreObservationRequest {
    pub fn into_record(self) -> ObservationRecord {
        let id = self.id.unwrap_or_else(|| {
            let payload = serde_json::json!({
                "namespace": self.namespace,
                "tool_name": self.tool_name,
                "resource_id": self.resource_id,
                "topic": self.topic,
                "content": self.content,
                "confidence": self.confidence,
            });
            stable_identifier("observation", &payload)
        });
        ObservationRecord {
            id,
            namespace: self.namespace,
            tool_name: self.tool_name,
            resource_id: self.resource_id,
            topic: self.topic,
            content: self.content,
            confidence: self.confidence,
            updated_at: Utc::now(),
            ttl_seconds: self.ttl_seconds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stable_identifier_deterministic() {
        let payload = serde_json::json!({"tool_name": "sqlite3", "topic": "schema"});
        let id1 = stable_identifier("fact", &payload);
        let id2 = stable_identifier("fact", &payload);
        assert_eq!(id1, id2);
        assert!(id1.starts_with("fact_"));
        assert_eq!(id1.len(), 5 + 12); // "fact_" + 12 hex chars
    }
}
