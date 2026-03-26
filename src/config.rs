use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub redis_url: String,
    pub memory_ttl_seconds: u64,
    pub default_namespace: String,
    pub max_context_items: usize,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            redis_url: env::var("ITERUM_REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379/0".into()),
            memory_ttl_seconds: env::var("ITERUM_MEMORY_TTL_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(7 * 24 * 60 * 60),
            default_namespace: env::var("ITERUM_DEFAULT_NAMESPACE")
                .unwrap_or_else(|_| "default".into()),
            max_context_items: env::var("ITERUM_MAX_CONTEXT_ITEMS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            port: env::var("ITERUM_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(8000),
        }
    }

    pub fn is_memory_store(&self) -> bool {
        self.redis_url.starts_with("memory://")
    }
}
