use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};

use crate::error::AppError;
use crate::extractors::ExtractorRegistry;
use crate::models::*;
use crate::routes::AppState;
use crate::store::Store;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/events/tool-result", post(handle_tool_result))
}

async fn handle_tool_result(
    State(retriever): State<AppState>,
    Json(event): Json<ToolResultEvent>,
) -> Result<Json<ToolResultResponse>, AppError> {
    let registry = ExtractorRegistry::default();
    let extracted = registry.extract(&event);
    let mut stored = Vec::new();

    for fact in extracted.facts {
        let record = retriever.store().put_fact(fact).await?;
        stored.push(StoredItem {
            record_type: "fact".into(),
            id: record.id,
        });
    }

    for constraint in extracted.constraints {
        let record = retriever.store().put_constraint(constraint).await?;
        stored.push(StoredItem {
            record_type: "constraint".into(),
            id: record.id,
        });
    }

    for fp in extracted.failure_patterns {
        let record = retriever.store().put_failure_pattern(fp).await?;
        stored.push(StoredItem {
            record_type: "failure_pattern".into(),
            id: record.id,
        });
    }

    for obs in extracted.observations {
        let record = retriever.store().put_observation(obs).await?;
        stored.push(StoredItem {
            record_type: "observation".into(),
            id: record.id,
        });
    }

    Ok(Json(ToolResultResponse { stored }))
}
