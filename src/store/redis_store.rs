use chrono::Utc;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;

use crate::error::AppError;
use crate::models::*;
use crate::store::Store;

fn segment(value: Option<&str>) -> &str {
    value.unwrap_or("*")
}

pub struct RedisStore {
    conn: ConnectionManager,
    ttl_seconds: u64,
}

impl RedisStore {
    pub fn new(conn: ConnectionManager, ttl_seconds: u64) -> Self {
        Self { conn, ttl_seconds }
    }

    // Key builders — same structure as Python
    fn fact_key(r: &FactRecord) -> String {
        format!(
            "iterum:fact:{}:{}:{}:{}",
            r.namespace,
            r.tool_name,
            segment(r.resource_id.as_deref()),
            r.id
        )
    }

    fn playbook_key(r: &PlaybookRecord) -> String {
        format!(
            "iterum:playbook:{}:{}:{}",
            r.namespace, r.tool_name, r.id
        )
    }

    fn observation_key(r: &ObservationRecord) -> String {
        format!(
            "iterum:observation:{}:{}:{}:{}",
            r.namespace,
            r.tool_name,
            segment(r.resource_id.as_deref()),
            r.id
        )
    }

    fn constraint_key(r: &ConstraintRecord) -> String {
        format!(
            "iterum:constraint:{}:{}:{}:{}",
            r.namespace,
            r.tool_name,
            segment(r.resource_id.as_deref()),
            r.id
        )
    }

    fn failure_pattern_key(r: &FailurePatternRecord) -> String {
        format!(
            "iterum:failure_pattern:{}:{}:{}:{}",
            r.namespace,
            r.tool_name,
            segment(r.resource_id.as_deref()),
            r.id
        )
    }

    // Index keys
    fn fact_tool_index(namespace: &str, tool_name: &str) -> String {
        format!("iterum:index:facts:{namespace}:{tool_name}")
    }

    fn fact_resource_index(namespace: &str, tool_name: &str, resource_id: Option<&str>) -> String {
        format!(
            "iterum:index:facts:{namespace}:{tool_name}:{}",
            segment(resource_id)
        )
    }

    fn playbook_tool_index(namespace: &str, tool_name: &str) -> String {
        format!("iterum:index:playbooks:{namespace}:{tool_name}")
    }

    fn playbook_family_index(
        namespace: &str,
        tool_name: &str,
        error_family: Option<&str>,
    ) -> String {
        format!(
            "iterum:index:playbooks:{namespace}:{tool_name}:{}",
            segment(error_family)
        )
    }

    fn observation_tool_index(namespace: &str, tool_name: &str) -> String {
        format!("iterum:index:observations:{namespace}:{tool_name}")
    }

    fn observation_resource_index(
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> String {
        format!(
            "iterum:index:observations:{namespace}:{tool_name}:{}",
            segment(resource_id)
        )
    }

    fn constraint_tool_index(namespace: &str, tool_name: &str) -> String {
        format!("iterum:index:constraints:{namespace}:{tool_name}")
    }

    fn constraint_resource_index(
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> String {
        format!(
            "iterum:index:constraints:{namespace}:{tool_name}:{}",
            segment(resource_id)
        )
    }

    fn fp_tool_index(namespace: &str, tool_name: &str) -> String {
        format!("iterum:index:failure_patterns:{namespace}:{tool_name}")
    }

    fn fp_resource_index(
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> String {
        format!(
            "iterum:index:failure_patterns:{namespace}:{tool_name}:{}",
            segment(resource_id)
        )
    }

    async fn get_records<T: serde::de::DeserializeOwned>(
        &self,
        keys: Vec<String>,
    ) -> Result<Vec<T>, AppError> {
        let mut conn = self.conn.clone();
        let mut records = Vec::new();
        for key in keys {
            let payload: Option<String> = conn.get(&key).await?;
            if let Some(data) = payload {
                if let Ok(record) = serde_json::from_str(&data) {
                    records.push(record);
                }
            }
        }
        Ok(records)
    }

    async fn collect_keys(
        &self,
        index_keys: Vec<String>,
    ) -> Result<Vec<String>, AppError> {
        let mut conn = self.conn.clone();
        let mut all_keys = std::collections::BTreeSet::new();
        for idx in index_keys {
            let members: Vec<String> = conn.smembers(&idx).await?;
            all_keys.extend(members);
        }
        Ok(all_keys.into_iter().collect())
    }
}

impl Store for RedisStore {
    async fn put_fact(&self, record: FactRecord) -> Result<FactRecord, AppError> {
        let mut conn = self.conn.clone();
        let key = Self::fact_key(&record);
        let data = serde_json::to_string(&record)?;
        conn.set::<_, _, ()>(&key, &data).await?;
        conn.sadd::<_, _, ()>(
            Self::fact_tool_index(&record.namespace, &record.tool_name),
            &key,
        )
        .await?;
        conn.sadd::<_, _, ()>(
            Self::fact_resource_index(
                &record.namespace,
                &record.tool_name,
                record.resource_id.as_deref(),
            ),
            &key,
        )
        .await?;
        Ok(record)
    }

    async fn list_facts(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<FactRecord>, AppError> {
        let mut indices = vec![Self::fact_tool_index(namespace, tool_name)];
        if let Some(rid) = resource_id {
            indices.push(Self::fact_resource_index(namespace, tool_name, Some(rid)));
            indices.push(Self::fact_resource_index(namespace, tool_name, None));
        }
        let keys = self.collect_keys(indices).await?;
        self.get_records(keys).await
    }

    async fn put_playbook(&self, record: PlaybookRecord) -> Result<PlaybookRecord, AppError> {
        let mut conn = self.conn.clone();
        let key = Self::playbook_key(&record);
        let data = serde_json::to_string(&record)?;
        conn.set::<_, _, ()>(&key, &data).await?;
        conn.sadd::<_, _, ()>(
            Self::playbook_tool_index(&record.namespace, &record.tool_name),
            &key,
        )
        .await?;
        conn.sadd::<_, _, ()>(
            Self::playbook_family_index(
                &record.namespace,
                &record.tool_name,
                record.error_family.as_deref(),
            ),
            &key,
        )
        .await?;
        Ok(record)
    }

    async fn list_playbooks(
        &self,
        namespace: &str,
        tool_name: &str,
        error_family: Option<&str>,
    ) -> Result<Vec<PlaybookRecord>, AppError> {
        let mut indices = vec![Self::playbook_tool_index(namespace, tool_name)];
        if let Some(ef) = error_family {
            indices.push(Self::playbook_family_index(namespace, tool_name, Some(ef)));
            indices.push(Self::playbook_family_index(namespace, tool_name, None));
        }
        let keys = self.collect_keys(indices).await?;
        self.get_records(keys).await
    }

    async fn put_observation(
        &self,
        record: ObservationRecord,
    ) -> Result<ObservationRecord, AppError> {
        let mut conn = self.conn.clone();
        let key = Self::observation_key(&record);
        let data = serde_json::to_string(&record)?;
        let ttl = record.ttl_seconds.unwrap_or(self.ttl_seconds);
        conn.set_ex::<_, _, ()>(&key, &data, ttl).await?;
        conn.sadd::<_, _, ()>(
            Self::observation_tool_index(&record.namespace, &record.tool_name),
            &key,
        )
        .await?;
        conn.sadd::<_, _, ()>(
            Self::observation_resource_index(
                &record.namespace,
                &record.tool_name,
                record.resource_id.as_deref(),
            ),
            &key,
        )
        .await?;
        Ok(record)
    }

    async fn list_observations(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<ObservationRecord>, AppError> {
        let mut indices = vec![Self::observation_tool_index(namespace, tool_name)];
        if let Some(rid) = resource_id {
            indices.push(Self::observation_resource_index(
                namespace,
                tool_name,
                Some(rid),
            ));
            indices.push(Self::observation_resource_index(namespace, tool_name, None));
        }
        let keys = self.collect_keys(indices).await?;
        self.get_records(keys).await
    }

    async fn put_constraint(
        &self,
        record: ConstraintRecord,
    ) -> Result<ConstraintRecord, AppError> {
        let mut conn = self.conn.clone();
        let key = Self::constraint_key(&record);
        let data = serde_json::to_string(&record)?;
        if let Some(ttl) = record.ttl_seconds {
            conn.set_ex::<_, _, ()>(&key, &data, ttl).await?;
        } else {
            conn.set_ex::<_, _, ()>(&key, &data, self.ttl_seconds).await?;
        }
        conn.sadd::<_, _, ()>(
            Self::constraint_tool_index(&record.namespace, &record.tool_name),
            &key,
        )
        .await?;
        conn.sadd::<_, _, ()>(
            Self::constraint_resource_index(
                &record.namespace,
                &record.tool_name,
                record.resource_id.as_deref(),
            ),
            &key,
        )
        .await?;
        Ok(record)
    }

    async fn list_constraints(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<ConstraintRecord>, AppError> {
        let mut indices = vec![Self::constraint_tool_index(namespace, tool_name)];
        if let Some(rid) = resource_id {
            indices.push(Self::constraint_resource_index(
                namespace,
                tool_name,
                Some(rid),
            ));
            indices.push(Self::constraint_resource_index(namespace, tool_name, None));
        }
        let keys = self.collect_keys(indices).await?;
        self.get_records(keys).await
    }

    async fn put_failure_pattern(
        &self,
        record: FailurePatternRecord,
    ) -> Result<FailurePatternRecord, AppError> {
        let mut conn = self.conn.clone();
        let key = Self::failure_pattern_key(&record);

        // Check if it already exists — if so, increment
        let existing: Option<String> = conn.get(&key).await?;
        let record = if let Some(data) = existing {
            if let Ok(mut existing_record) = serde_json::from_str::<FailurePatternRecord>(&data) {
                existing_record.occurrence_count += 1;
                existing_record.last_seen = Utc::now();
                existing_record
            } else {
                record
            }
        } else {
            record
        };

        let data = serde_json::to_string(&record)?;
        if let Some(ttl) = record.ttl_seconds {
            conn.set_ex::<_, _, ()>(&key, &data, ttl).await?;
        } else {
            conn.set_ex::<_, _, ()>(&key, &data, self.ttl_seconds).await?;
        }
        conn.sadd::<_, _, ()>(
            Self::fp_tool_index(&record.namespace, &record.tool_name),
            &key,
        )
        .await?;
        conn.sadd::<_, _, ()>(
            Self::fp_resource_index(
                &record.namespace,
                &record.tool_name,
                record.resource_id.as_deref(),
            ),
            &key,
        )
        .await?;
        Ok(record)
    }

    async fn list_failure_patterns(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<FailurePatternRecord>, AppError> {
        let mut indices = vec![Self::fp_tool_index(namespace, tool_name)];
        if let Some(rid) = resource_id {
            indices.push(Self::fp_resource_index(namespace, tool_name, Some(rid)));
            indices.push(Self::fp_resource_index(namespace, tool_name, None));
        }
        let keys = self.collect_keys(indices).await?;
        self.get_records(keys).await
    }
}
