pub mod context;
pub mod debug;
pub mod events;

use std::sync::Arc;

use axum::Router;

use crate::retrieval::ContextRetriever;
use crate::store::memory::InMemoryStore;

pub type AppState = Arc<ContextRetriever<InMemoryStore>>;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .merge(context::router())
        .merge(events::router())
        .merge(debug::router())
        .with_state(state)
}
