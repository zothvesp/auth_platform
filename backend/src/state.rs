use crate::{
    config::AppConfig,
    db::{Database, RedisPool},
};
use std::sync::Arc;

/// Shared application state injected into every Axum handler.
/// Cloning is cheap — Arc for config, pool handles are reference-counted internally.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: RedisPool,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(db: Database, redis: RedisPool, config: AppConfig) -> Self {
        Self {
            db,
            redis,
            config: Arc::new(config),
        }
    }
}
