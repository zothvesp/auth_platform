use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppResult,
    middleware::auth::AuthUser,
    services::{audit, rbac},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_permissions).post(create_permission))
        .route("/:id", get(get_permission).put(update_permission).delete(delete_permission))
}

#[derive(Deserialize)]
pub struct CreatePermissionRequest {
    pub resource: String,
    pub action: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct UpdatePermissionRequest {
    pub resource: String,
    pub action: String,
    pub description: String,
}

pub async fn list_permissions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("permissions:read")?;
    Ok(Json(rbac::list_permissions(&state).await?))
}

pub async fn get_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("permissions:read")?;
    Ok(Json(rbac::get_permission(&state, id).await?))
}

pub async fn create_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<CreatePermissionRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("permissions:manage")?;
    let perm = rbac::create_permission(&state, &req.resource, &req.action, &req.description).await?;

    audit::record(
        &state, Some(auth.user_id), Some(&auth.email),
        "permission.create", "permission", Some(&perm.id.to_string()),
        &crate::utils::extract_ip(&headers), &crate::utils::extract_ua(&headers),
        true, Some(serde_json::json!({ "resource": req.resource, "action": req.action })),
    ).await;

    Ok((StatusCode::CREATED, Json(perm)))
}

pub async fn update_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePermissionRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("permissions:manage")?;
    let perm = rbac::update_permission(&state, id, &req.resource, &req.action, &req.description).await?;

    audit::record(
        &state, Some(auth.user_id), Some(&auth.email),
        "permission.update", "permission", Some(&id.to_string()),
        &crate::utils::extract_ip(&headers), &crate::utils::extract_ua(&headers),
        true, Some(serde_json::json!({ "resource": req.resource, "action": req.action })),
    ).await;

    Ok(Json(perm))
}

pub async fn delete_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("permissions:manage")?;
    rbac::delete_permission(&state, id).await?;

    audit::record(
        &state, Some(auth.user_id), Some(&auth.email),
        "permission.delete", "permission", Some(&id.to_string()),
        &crate::utils::extract_ip(&headers), &crate::utils::extract_ua(&headers),
        true, None,
    ).await;

    Ok(StatusCode::NO_CONTENT)
}
