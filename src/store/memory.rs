use std::collections::HashMap;
use std::sync::RwLock;

use chrono::Utc;

use crate::error::AppError;
use crate::models::*;
use crate::store::Store;

const MAX_FAILURE_PATTERNS_PER_RESOURCE: usize = 20;

type Key = (String, String, String); // (namespace, tool_name, id)

pub struct InMemoryStore {
    facts: RwLock<HashMap<Key, FactRecord>>,
    playbooks: RwLock<HashMap<Key, PlaybookRecord>>,
    observations: RwLock<HashMap<Key, ObservationRecord>>,
    constraints: RwLock<HashMap<Key, ConstraintRecord>>,
    failure_patterns: RwLock<HashMap<Key, FailurePatternRecord>>,
    default_ttl: u64,
}

impl InMemoryStore {
    pub fn new(default_ttl: u64) -> Self {
        Self {
            facts: RwLock::new(HashMap::new()),
            playbooks: RwLock::new(HashMap::new()),
            observations: RwLock::new(HashMap::new()),
            constraints: RwLock::new(HashMap::new()),
            failure_patterns: RwLock::new(HashMap::new()),
            default_ttl,
        }
    }

    fn is_expired_observation(record: &ObservationRecord, default_ttl: u64) -> bool {
        let ttl = record.ttl_seconds.unwrap_or(default_ttl);
        let elapsed = (Utc::now() - record.updated_at).num_seconds();
        elapsed > ttl as i64
    }

    fn is_expired_constraint(record: &ConstraintRecord, default_ttl: u64) -> bool {
        if let Some(ttl) = record.ttl_seconds {
            let elapsed = (Utc::now() - record.updated_at).num_seconds();
            return elapsed > ttl as i64;
        }
        let elapsed = (Utc::now() - record.updated_at).num_seconds();
        elapsed > default_ttl as i64
    }

    fn is_expired_failure_pattern(record: &FailurePatternRecord, default_ttl: u64) -> bool {
        if let Some(ttl) = record.ttl_seconds {
            let elapsed = (Utc::now() - record.last_seen).num_seconds();
            return elapsed > ttl as i64;
        }
        let elapsed = (Utc::now() - record.last_seen).num_seconds();
        elapsed > default_ttl as i64
    }
}

impl Store for InMemoryStore {
    async fn put_fact(&self, record: FactRecord) -> Result<FactRecord, AppError> {
        let key = (
            record.namespace.clone(),
            record.tool_name.clone(),
            record.id.clone(),
        );
        self.facts
            .write()
            .map_err(|e| AppError::Store(e.to_string()))?
            .insert(key, record.clone());
        Ok(record)
    }

    async fn list_facts(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<FactRecord>, AppError> {
        let guard = self
            .facts
            .read()
            .map_err(|e| AppError::Store(e.to_string()))?;
        Ok(guard
            .values()
            .filter(|r| {
                r.namespace == namespace
                    && r.tool_name == tool_name
                    && (resource_id.is_none()
                        || r.resource_id.as_deref() == resource_id
                        || r.resource_id.is_none())
            })
            .cloned()
            .collect())
    }

    async fn put_playbook(&self, record: PlaybookRecord) -> Result<PlaybookRecord, AppError> {
        let key = (
            record.namespace.clone(),
            record.tool_name.clone(),
            record.id.clone(),
        );
        self.playbooks
            .write()
            .map_err(|e| AppError::Store(e.to_string()))?
            .insert(key, record.clone());
        Ok(record)
    }

    async fn list_playbooks(
        &self,
        namespace: &str,
        tool_name: &str,
        error_family: Option<&str>,
    ) -> Result<Vec<PlaybookRecord>, AppError> {
        let guard = self
            .playbooks
            .read()
            .map_err(|e| AppError::Store(e.to_string()))?;
        Ok(guard
            .values()
            .filter(|r| {
                r.namespace == namespace
                    && r.tool_name == tool_name
                    && (error_family.is_none()
                        || r.error_family.as_deref() == error_family
                        || r.error_family.is_none())
            })
            .cloned()
            .collect())
    }

    async fn put_observation(
        &self,
        record: ObservationRecord,
    ) -> Result<ObservationRecord, AppError> {
        let key = (
            record.namespace.clone(),
            record.tool_name.clone(),
            record.id.clone(),
        );
        self.observations
            .write()
            .map_err(|e| AppError::Store(e.to_string()))?
            .insert(key, record.clone());
        Ok(record)
    }

    async fn list_observations(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<ObservationRecord>, AppError> {
        let guard = self
            .observations
            .read()
            .map_err(|e| AppError::Store(e.to_string()))?;
        Ok(guard
            .values()
            .filter(|r| {
                !Self::is_expired_observation(r, self.default_ttl)
                    && r.namespace == namespace
                    && r.tool_name == tool_name
                    && (resource_id.is_none()
                        || r.resource_id.as_deref() == resource_id
                        || r.resource_id.is_none())
            })
            .cloned()
            .collect())
    }

    async fn put_constraint(
        &self,
        record: ConstraintRecord,
    ) -> Result<ConstraintRecord, AppError> {
        let key = (
            record.namespace.clone(),
            record.tool_name.clone(),
            record.id.clone(),
        );
        self.constraints
            .write()
            .map_err(|e| AppError::Store(e.to_string()))?
            .insert(key, record.clone());
        Ok(record)
    }

    async fn list_constraints(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<ConstraintRecord>, AppError> {
        let guard = self
            .constraints
            .read()
            .map_err(|e| AppError::Store(e.to_string()))?;
        Ok(guard
            .values()
            .filter(|r| {
                !Self::is_expired_constraint(r, self.default_ttl)
                    && r.namespace == namespace
                    && r.tool_name == tool_name
                    && (resource_id.is_none()
                        || r.resource_id.as_deref() == resource_id
                        || r.resource_id.is_none())
            })
            .cloned()
            .collect())
    }

    async fn put_failure_pattern(
        &self,
        record: FailurePatternRecord,
    ) -> Result<FailurePatternRecord, AppError> {
        let mut guard = self
            .failure_patterns
            .write()
            .map_err(|e| AppError::Store(e.to_string()))?;

        let key = (
            record.namespace.clone(),
            record.tool_name.clone(),
            record.id.clone(),
        );

        // If it already exists, increment occurrence_count
        if let Some(existing) = guard.get_mut(&key) {
            existing.occurrence_count += 1;
            existing.last_seen = Utc::now();
            return Ok(existing.clone());
        }

        // FIFO eviction: remove oldest if over limit for this resource
        let resource_key = (
            record.namespace.clone(),
            record.tool_name.clone(),
            record.resource_id.clone().unwrap_or_default(),
        );
        let same_resource: Vec<Key> = guard
            .iter()
            .filter(|(_, r)| {
                r.namespace == resource_key.0
                    && r.tool_name == resource_key.1
                    && r.resource_id.as_deref().unwrap_or("") == resource_key.2
            })
            .map(|(k, _)| k.clone())
            .collect();

        if same_resource.len() >= MAX_FAILURE_PATTERNS_PER_RESOURCE {
            // Find oldest by first_seen
            if let Some(oldest_key) = same_resource
                .iter()
                .min_by_key(|k| guard.get(*k).map(|r| r.first_seen))
                .cloned()
            {
                guard.remove(&oldest_key);
            }
        }

        guard.insert(key, record.clone());
        Ok(record)
    }

    async fn list_failure_patterns(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<FailurePatternRecord>, AppError> {
        let guard = self
            .failure_patterns
            .read()
            .map_err(|e| AppError::Store(e.to_string()))?;
        Ok(guard
            .values()
            .filter(|r| {
                !Self::is_expired_failure_pattern(r, self.default_ttl)
                    && r.namespace == namespace
                    && r.tool_name == tool_name
                    && (resource_id.is_none()
                        || r.resource_id.as_deref() == resource_id
                        || r.resource_id.is_none())
            })
            .cloned()
            .collect())
    }
}
