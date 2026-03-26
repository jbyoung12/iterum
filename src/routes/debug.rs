use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};

use crate::error::AppError;
use crate::models::*;
use crate::routes::AppState;
use crate::store::Store;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/v1/debug/facts", get(debug_facts))
        .route("/v1/debug/playbooks", get(debug_playbooks))
        .route("/v1/debug/observations", get(debug_observations))
        .route("/v1/debug/constraints", get(debug_constraints))
        .route(
            "/v1/debug/failure-patterns",
            get(debug_failure_patterns),
        )
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn debug_facts(
    State(retriever): State<AppState>,
    Query(q): Query<DebugQuery>,
) -> Result<Json<Vec<FactRecord>>, AppError> {
    let records = retriever
        .store()
        .list_facts(&q.namespace, &q.tool_name, q.resource_id.as_deref())
        .await?;
    Ok(Json(records))
}

async fn debug_playbooks(
    State(retriever): State<AppState>,
    Query(q): Query<DebugQuery>,
) -> Result<Json<Vec<PlaybookRecord>>, AppError> {
    let records = retriever
        .store()
        .list_playbooks(&q.namespace, &q.tool_name, q.error_family.as_deref())
        .await?;
    Ok(Json(records))
}

async fn debug_observations(
    State(retriever): State<AppState>,
    Query(q): Query<DebugQuery>,
) -> Result<Json<Vec<ObservationRecord>>, AppError> {
    let records = retriever
        .store()
        .list_observations(&q.namespace, &q.tool_name, q.resource_id.as_deref())
        .await?;
    Ok(Json(records))
}

async fn debug_constraints(
    State(retriever): State<AppState>,
    Query(q): Query<DebugQuery>,
) -> Result<Json<Vec<ConstraintRecord>>, AppError> {
    let records = retriever
        .store()
        .list_constraints(&q.namespace, &q.tool_name, q.resource_id.as_deref())
        .await?;
    Ok(Json(records))
}

async fn debug_failure_patterns(
    State(retriever): State<AppState>,
    Query(q): Query<DebugQuery>,
) -> Result<Json<Vec<FailurePatternRecord>>, AppError> {
    let records = retriever
        .store()
        .list_failure_patterns(&q.namespace, &q.tool_name, q.resource_id.as_deref())
        .await?;
    Ok(Json(records))
}
