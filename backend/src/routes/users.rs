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
    repositories::{base::BaseRepository, UserRepository},
    services,
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
    let dto = services::auth::build_user_dto(&state, auth.user_id).await?;
    Ok(Json(dto))
}

pub async fn update_me(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateMeRequest>,
) -> AppResult<impl IntoResponse> {
    if let (Some(name), avatar) = (req.display_name.as_deref(), req.avatar_url.as_deref()) {
        UserRepository::update_profile(&state.db.pool, auth.user_id, name, avatar).await?;
    }
    Ok(Json(
        services::auth::build_user_dto(&state, auth.user_id).await?,
    ))
}

pub async fn change_password(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<impl IntoResponse> {
    let user = UserRepository::new(&state.db.pool)
        .get(auth.user_id)
        .await?;
    services::auth::verify_password(
        &req.current_password,
        user.password_hash.as_deref().unwrap_or(""),
    )?;
    // Validate new password against DB policy
    let policy = services::config::password_policy(&state).await;
    let violations = policy.validate(&req.new_password);
    if !violations.is_empty() {
        let mut d = std::collections::HashMap::new();
        d.insert("new_password".to_string(), violations);
        return Err(crate::error::AppError::Validation(d));
    }
    let hash = services::auth::hash_password(&req.new_password)?;
    UserRepository::update_password(&state.db.pool, auth.user_id, &hash).await?;
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
    State(_state): State<AppState>,
    _auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    // Placeholder — sessions table managed separately
    Ok(Json(serde_json::json!([])))
}

pub async fn revoke_session(
    State(_state): State<AppState>,
    _auth: AuthUser,
    Path(_session_id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    // Placeholder — revoke by deleting from sessions table
    Ok(StatusCode::NO_CONTENT)
}
