use axum::{
    extract::{Json, Path, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::AppResult,
    middleware::auth::AuthUser,
    models::Role,
    services::{audit, rbac},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_roles).post(create_role))
        .route("/:id", get(get_role).put(update_role).delete(delete_role))
        .route("/:id/permissions", get(get_role_permissions))
        .route(
            "/:id/permissions/:perm_id",
            post(assign_permission).delete(remove_permission),
        )
}

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
    pub description: String,
    pub parent_role_id: Option<Uuid>,
    pub permission_ids: Option<Vec<Uuid>>,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub description: Option<String>,
    pub permission_ids: Option<Vec<Uuid>>,
}

#[derive(Serialize)]
pub struct RoleResponse {
    #[serde(flatten)]
    pub role: Role,
    pub permissions: Vec<crate::models::Permission>,
}

pub async fn list_roles(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:read")?;
    Ok(Json(rbac::list_roles(&state).await?))
}

pub async fn get_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:read")?;
    let role = rbac::get_role(&state, id).await?;
    let permissions = rbac::get_role_permissions(&state, id).await?;
    Ok(Json(RoleResponse { role, permissions }))
}

pub async fn create_role(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<CreateRoleRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:create")?;
    rbac::validate_role_name(&req.name)?;
    let perm_ids = req.permission_ids.unwrap_or_default();
    let role = rbac::create_role(
        &state,
        &req.name,
        &req.description,
        req.parent_role_id,
        &perm_ids,
    )
    .await?;

    audit::record(
        &state, Some(auth.user_id), Some(&auth.email),
        "role.create", "role", Some(&role.id.to_string()),
        &crate::utils::extract_ip(&headers), &crate::utils::extract_ua(&headers),
        true, None,
    ).await;

    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn update_role(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRoleRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:update")?;
    let (updated, permissions) = rbac::update_role(&state, id, req.description.as_deref(), req.permission_ids).await?;

    audit::record(
        &state, Some(auth.user_id), Some(&auth.email),
        "role.update", "role", Some(&id.to_string()),
        &crate::utils::extract_ip(&headers), &crate::utils::extract_ua(&headers),
        true, None,
    ).await;

    Ok(Json(RoleResponse { role: updated, permissions }))
}

pub async fn delete_role(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:delete")?;
    rbac::delete_role(&state, id).await?;

    audit::record(
        &state, Some(auth.user_id), Some(&auth.email),
        "role.delete", "role", Some(&id.to_string()),
        &crate::utils::extract_ip(&headers), &crate::utils::extract_ua(&headers),
        true, None,
    ).await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_role_permissions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:read")?;
    Ok(Json(rbac::get_role_permissions(&state, id).await?))
}

pub async fn assign_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((role_id, perm_id)): Path<(Uuid, Uuid)>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:update")?;
    rbac::assign_permission(&state, role_id, perm_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((role_id, perm_id)): Path<(Uuid, Uuid)>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:update")?;
    rbac::remove_permission(&state, role_id, perm_id).await?;
    Ok(StatusCode::NO_CONTENT)
}


