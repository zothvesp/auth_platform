use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppResult,
    middleware::auth::AuthUser,
    repositories::{SessionRepository, UserRepository},
    services::{admin, auth, gdpr, session},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // Current user
        .route("/me", get(get_me).put(update_me))
        .route("/me/change-password", post(change_password))
        .route("/me/login-history", get(login_history))
        .route("/me/sessions", get(active_sessions))
        .route("/me/sessions/:id", delete(revoke_session))
        // OAuth account linking
        .route("/me/oauth-accounts", get(list_oauth_accounts))
        .route("/me/oauth-accounts/:provider", delete(unlink_oauth_account))
        // GDPR
        .route("/me/export", get(export_my_data))
        .route("/me/delete", delete(delete_my_account))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMeRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

pub async fn get_me(State(state): State<AppState>, auth: AuthUser) -> AppResult<impl IntoResponse> {
    let dto = auth::build_user_dto(&state, auth.user_id).await?;
    Ok(Json(dto))
}

pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateMeRequest>,
) -> AppResult<impl IntoResponse> {
    let result = admin::update_user(
        &state,
        auth.user_id,
        req.display_name.as_deref(),
        req.avatar_url.as_deref(),
    )
    .await?;
    Ok(Json(result))
}

pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<impl IntoResponse> {
    auth::change_password(&state, auth.user_id, &req.current_password, &req.new_password).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn login_history(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    let history = UserRepository::new(&state.db.pool)
        .login_history(auth.user_id, 50)
        .await?;
    Ok(Json(history))
}

pub async fn active_sessions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    Ok(Json(
        SessionRepository::new(&state.db.pool)
            .find_active_by_user(auth.user_id)
            .await?,
    ))
}

pub async fn revoke_session(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(session_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    session::revoke(&state, auth.user_id, session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_oauth_accounts(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    let accounts = crate::services::oauth::list_linked_accounts(&state, auth.user_id).await?;
    Ok(Json(accounts))
}

pub async fn unlink_oauth_account(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(provider): Path<String>,
) -> AppResult<impl IntoResponse> {
    crate::services::oauth::unlink_account(&state, auth.user_id, &provider).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ─── GDPR ─────────────────────────────────────────────────────────────────────

/// Export all data for the authenticated user (GDPR Article 20).
pub async fn export_my_data(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    let data = gdpr::export_user_data(&state, auth.user_id).await?;
    Ok(Json(data))
}

/// Soft-delete the authenticated user's account (GDPR Article 17).
/// Anonymizes PII and revokes all sessions/tokens.
pub async fn delete_my_account(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    gdpr::soft_delete_user(&state, auth.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
