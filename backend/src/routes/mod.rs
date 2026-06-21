pub mod admin;
pub mod auth;
pub mod config;
pub mod oauth;
pub mod oauth_apps;
pub mod oauth_provider;
pub mod permissions;
pub mod roles;
pub mod saml;
pub mod users;

use crate::state::AppState;
use axum::Router;

pub fn build_v1_router(state: AppState) -> Router<AppState> {
    let auth_routes = auth::router().layer(axum::middleware::from_fn_with_state(
        state.clone(),
        crate::middleware::rate_limit::ip_rate_limit,
    ));

    let admin_routes = admin::router()
        .merge(oauth_apps::router());

    Router::new()
        .nest("/auth", auth_routes)
        .nest("/auth/oauth", oauth::router())
        .nest("/auth", saml::router())
        .nest("/config", config::router())
        .nest("/users", users::router())
        .nest("/roles", roles::router())
        .nest("/permissions", permissions::router())
        .nest("/admin", admin_routes)
}

/// Routes that live outside /api/v1 (OIDC discovery, OAuth endpoints)
pub fn build_provider_router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/", oauth_provider::router())
        .with_state(state)
}
