#![recursion_limit = "512"]

use authforge_backend::{
    config::AppConfig,
    db::{Database, RedisPool},
    services,
    state::AppState,
    vault,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env()?;

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "authforge=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    info!("Connecting to PostgreSQL...");
    let db = Database::connect(&config.database_url).await?;
    db.run_migrations().await?;

    let redis = RedisPool::connect_or_noop(&config.redis_url).await;
    let vault_impl = vault::create_vault(db.pool.clone());
    vault_impl.ensure_signing_key().await?;
    let state = AppState::new(db, redis, config.clone(), vault_impl);
    services::rbac::seed_defaults(&state).await?;

    let app = authforge_backend::build_router(state, &config);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("AuthForge listening on http://{}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
