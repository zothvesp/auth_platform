#![recursion_limit = "256"]

pub mod config;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;
pub mod utils;
pub mod vault;

use crate::{
    config::AppConfig,
    state::AppState,
};
use axum::{
    http::{header, HeaderName, HeaderValue, Method},
    Router,
};
use std::time::Duration;
use tower_http::{
    cors::CorsLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};

pub fn build_router(state: AppState, config: &AppConfig) -> Router {
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
            HeaderName::from_static("x-csrf-token"),
        ])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600));

    Router::new()
        .route("/health", axum::routing::get(|| async { "OK" }))
        .nest("/api/v1", routes::build_v1_router(state.clone()))
        .merge(routes::build_provider_router(state.clone()))
        .layer(middleware::validation::body_limit_layer())
        .layer(axum::middleware::from_fn(
            middleware::validation::validate_content_type,
        ))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(middleware::csp::csp_headers))
        .layer(axum::middleware::from_fn(middleware::csrf::csrf_protection))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(cors)
        .with_state(state)
}
