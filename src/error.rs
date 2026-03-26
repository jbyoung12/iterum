use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("store error: {0}")]
    Store(String),

    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Store(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Serialization(_) => StatusCode::BAD_REQUEST,
        };
        (status, self.to_string()).into_response()
    }
}
