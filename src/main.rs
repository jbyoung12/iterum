use std::sync::Arc;

use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use iterum::config::Config;
use iterum::retrieval::ContextRetriever;
use iterum::routes::build_router;
use iterum::store::memory::InMemoryStore;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env();
    tracing::info!(
        redis_url = %config.redis_url,
        port = config.port,
        "starting iterum"
    );

    let store = InMemoryStore::new(config.memory_ttl_seconds);
    let retriever = Arc::new(ContextRetriever::new(store, config.max_context_items));

    let app = build_router(retriever).layer(TraceLayer::new_for_http());

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
