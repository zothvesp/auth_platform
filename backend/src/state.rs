use crate::{
    config::AppConfig,
    db::{Database, RedisPool},
    vault::VaultRepository,
};
use std::sync::Arc;

/// Shared application state injected into every Axum handler.
/// Cloning is cheap — Arc for config, pool handles are reference-counted internally.
#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub redis: RedisPool,
    pub config: Arc<AppConfig>,
    pub vault: Arc<dyn VaultRepository>,
}

impl AppState {
    pub fn new(db: Database, redis: RedisPool, config: AppConfig, vault: Box<dyn VaultRepository>) -> Self {
        Self {
            db,
            redis,
            config: Arc::new(config),
            vault: Arc::from(vault),
        }
    }
}
