pub mod memory;
pub mod redis_store;

use crate::error::AppError;
use crate::models::*;

#[allow(async_fn_in_trait)]
pub trait Store: Send + Sync + 'static {
    // Facts
    async fn put_fact(&self, record: FactRecord) -> Result<FactRecord, AppError>;
    async fn list_facts(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<FactRecord>, AppError>;

    // Playbooks
    async fn put_playbook(&self, record: PlaybookRecord) -> Result<PlaybookRecord, AppError>;
    async fn list_playbooks(
        &self,
        namespace: &str,
        tool_name: &str,
        error_family: Option<&str>,
    ) -> Result<Vec<PlaybookRecord>, AppError>;

    // Observations
    async fn put_observation(
        &self,
        record: ObservationRecord,
    ) -> Result<ObservationRecord, AppError>;
    async fn list_observations(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<ObservationRecord>, AppError>;

    // Constraints
    async fn put_constraint(
        &self,
        record: ConstraintRecord,
    ) -> Result<ConstraintRecord, AppError>;
    async fn list_constraints(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<ConstraintRecord>, AppError>;

    // Failure Patterns
    async fn put_failure_pattern(
        &self,
        record: FailurePatternRecord,
    ) -> Result<FailurePatternRecord, AppError>;
    async fn list_failure_patterns(
        &self,
        namespace: &str,
        tool_name: &str,
        resource_id: Option<&str>,
    ) -> Result<Vec<FailurePatternRecord>, AppError>;
}
