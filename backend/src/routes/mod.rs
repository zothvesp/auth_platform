pub mod admin;
pub mod auth;
pub mod config;
pub mod oauth;
pub mod permissions;
pub mod roles;
pub mod users;

use crate::state::AppState;
use axum::Router;

pub fn build_v1_router(_state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/auth/oauth", oauth::router())
        .nest("/config", config::router())
        .nest("/users", users::router())
        .nest("/roles", roles::router())
        .nest("/permissions", permissions::router())
        .nest("/admin", admin::router())
}
