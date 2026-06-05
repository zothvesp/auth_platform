use axum::{
    extract::{Json, State},
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::{
    error::{AppError, AppResult},
    models::config::keys,
    repositories::ConfigRepository,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:provider", get(oauth_redirect))
        .route("/:provider/callback", post(oauth_callback))
}

#[derive(Deserialize)]
pub struct CallbackRequest {
    pub code: String,
    pub state: String,
}

pub async fn oauth_redirect(
    State(state): State<AppState>,
    axum::extract::Path(provider): axum::extract::Path<String>,
) -> AppResult<axum::Json<serde_json::Value>> {
    // Check feature flag from DB
    let flag_key = match provider.as_str() {
        "google" => keys::OAUTH_GOOGLE_ENABLED,
        "github" => keys::OAUTH_GITHUB_ENABLED,
        "microsoft" => keys::OAUTH_MICROSOFT_ENABLED,
        _ => {
            return Err::<axum::Json<serde_json::Value>, _>(AppError::NotFound(
                "oauth provider".to_string(),
            ))
        }
    };

    let enabled = ConfigRepository::new(&state.db.pool)
        .get_bool(flag_key, false)
        .await;

    if !enabled {
        return Err(AppError::Forbidden);
    }

    // TODO Phase 3: build actual OAuth URL with PKCE
    Err::<axum::Json<serde_json::Value>, _>(AppError::Internal(anyhow::anyhow!(
        "OAuth not yet implemented"
    )))
}

pub async fn oauth_callback(
    State(_state): State<AppState>,
    axum::extract::Path(_provider): axum::extract::Path<String>,
    Json(_req): Json<CallbackRequest>,
) -> AppResult<axum::Json<serde_json::Value>> {
    // TODO Phase 3: exchange code, upsert user, issue tokens
    Err::<axum::Json<serde_json::Value>, _>(AppError::Internal(anyhow::anyhow!(
        "OAuth not yet implemented"
    )))
}
