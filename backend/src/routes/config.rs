use axum::{
    extract::{Path, State},
    routing::{get, put},
    Json, Router,
};
use serde::Deserialize;

use crate::{
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
    repositories::ConfigRepository,
    services,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // Public — no auth required; frontend fetches this on boot
        .route("/public", get(public_config))
        // Admin — requires settings:manage permission
        .route("/", get(list_all_config))
        .route("/:key", put(update_config))
}

/// GET /api/v1/config/public
/// Returns password policy, validation rules, and feature flags.
/// Safe for unauthenticated clients — no secrets, only UI-relevant settings.
pub async fn public_config(
    State(state): State<AppState>,
) -> AppResult<impl axum::response::IntoResponse> {
    let config = services::config::public_config(&state).await?;
    Ok(Json(config))
}

/// GET /api/v1/config
/// Admin: returns all config rows including non-public settings.
pub async fn list_all_config(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl axum::response::IntoResponse> {
    auth.require_permission("settings:manage")?;
    let rows = ConfigRepository::new(&state.db.pool).load_all().await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
pub struct UpdateConfigRequest {
    pub value: String,
}

/// PUT /api/v1/config/:key
/// Admin: update a single config value. Returns updated row.
pub async fn update_config(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(key): Path<String>,
    Json(payload): Json<UpdateConfigRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    auth.require_permission("settings:manage")?;

    // Safety: block setting secret values via the API
    const BLOCKED_KEYS: &[&str] = &[
        "jwt_private_key",
        "jwt_public_key",
        "smtp_password",
        "google_client_secret",
        "github_client_secret",
        "microsoft_client_secret",
    ];
    if BLOCKED_KEYS.iter().any(|k| key.contains(k)) {
        return Err(AppError::Forbidden);
    }

    ConfigRepository::set(&state.db.pool, &key, &payload.value).await?;

    // Return the updated row for optimistic UI update
    let value = ConfigRepository::new(&state.db.pool)
        .get(&key)
        .await?
        .ok_or_else(|| AppError::NotFound("config key".to_string()))?;

    Ok(Json(serde_json::json!({ "key": key, "value": value })))
}
