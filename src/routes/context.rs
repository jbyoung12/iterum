use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::error::AppError;
use crate::models::*;
use crate::routes::AppState;
use crate::store::Store;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/context/retrieve", post(retrieve_context))
        .route("/v1/context/facts", post(store_fact))
        .route("/v1/context/playbooks", post(store_playbook))
        .route("/v1/context/observations", post(store_observation))
}

async fn retrieve_context(
    State(retriever): State<AppState>,
    Json(request): Json<ContextRetrieveRequest>,
) -> Result<Json<ContextRetrieveResponse>, AppError> {
    let response = retriever.retrieve(&request).await?;
    Ok(Json(response))
}

async fn store_fact(
    State(retriever): State<AppState>,
    Json(request): Json<StoreFactRequest>,
) -> Result<Json<FactRecord>, AppError> {
    let record = request.into_record();
    let stored = retriever.store().put_fact(record).await?;
    Ok(Json(stored))
}

async fn store_playbook(
    State(retriever): State<AppState>,
    Json(request): Json<StorePlaybookRequest>,
) -> Result<Json<PlaybookRecord>, AppError> {
    let record = request.into_record();
    let stored = retriever.store().put_playbook(record).await?;
    Ok(Json(stored))
}

async fn store_observation(
    State(retriever): State<AppState>,
    Json(request): Json<StoreObservationRequest>,
) -> Result<Json<ObservationRecord>, AppError> {
    let record = request.into_record();
    let stored = retriever.store().put_observation(record).await?;
    Ok(Json(stored))
}
