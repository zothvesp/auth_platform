mod config;
mod db;
mod error;
mod middleware;
mod models;
mod repositories;
mod routes;
mod services;
mod state;
mod utils;

use crate::{
    config::AppConfig,
    db::{Database, RedisPool},
    state::AppState,
};
use axum::{
    http::{header, HeaderValue, Method},
    Router,
};
use std::time::Duration;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
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
    let state = AppState::new(db, redis, config.clone());
    services::rbac::seed_defaults(&state).await?;

    let app = build_router(state, &config);
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("AuthForge listening on http://{}", addr);
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}

fn build_router(state: AppState, config: &AppConfig) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            config
                .allowed_origins
                .iter()
                .filter_map(|o| o.parse::<HeaderValue>().ok())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::ORIGIN,
            header::COOKIE,
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));

    Router::new()
        .route("/health", axum::routing::get(|| async { "OK" }))
        .nest("/api/v1", routes::build_v1_router(state.clone()))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(TraceLayer::new_for_http())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(cors)
        .with_state(state)
}
