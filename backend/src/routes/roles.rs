use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
    models::Role,
    repositories::{base::BaseRepository, PermissionRepository, RoleRepository},
    services,
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
    Ok(Json(services::rbac::list_roles(&state).await?))
}

pub async fn get_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:read")?;
    let role = services::rbac::get_role(&state, id).await?;
    let permissions = RoleRepository::new(&state.db.pool)
        .find_permissions(id)
        .await?;
    Ok(Json(RoleResponse { role, permissions }))
}

pub async fn create_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateRoleRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:create")?;
    validate_role_name(&req.name)?;
    let perm_ids = req.permission_ids.unwrap_or_default();
    let role = services::rbac::create_role(
        &state,
        &req.name,
        &req.description,
        req.parent_role_id,
        &perm_ids,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn update_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateRoleRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:update")?;
    services::rbac::get_role(&state, id).await?; // 404 check
    let pool = &state.db.pool;
    let mut tx = pool.begin().await?;
    if let Some(desc) = &req.description {
        RoleRepository::update_description(&mut *tx, id, desc).await?;
    }
    if let Some(perm_ids) = req.permission_ids {
        RoleRepository::remove_all_permissions(&mut *tx, id).await?;
        for perm_id in perm_ids {
            RoleRepository::assign_permission(&mut *tx, id, perm_id).await?;
        }
    }
    tx.commit().await?;
    let updated = services::rbac::get_role(&state, id).await?;
    let permissions = RoleRepository::new(pool).find_permissions(id).await?;
    Ok(Json(RoleResponse {
        role: updated,
        permissions,
    }))
}

pub async fn delete_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:delete")?;
    services::rbac::delete_role(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_role_permissions(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:read")?;
    Ok(Json(
        RoleRepository::new(&state.db.pool)
            .find_permissions(id)
            .await?,
    ))
}

pub async fn assign_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((role_id, perm_id)): Path<(Uuid, Uuid)>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:update")?;
    services::rbac::get_role(&state, role_id).await?;
    PermissionRepository::new(&state.db.pool)
        .get(perm_id)
        .await?;
    RoleRepository::assign_permission(&state.db.pool, role_id, perm_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((role_id, perm_id)): Path<(Uuid, Uuid)>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("roles:update")?;
    RoleRepository::remove_permission(&state.db.pool, role_id, perm_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn validate_role_name(name: &str) -> AppResult<()> {
    if name.len() < 2
        || name.len() > 50
        || !name.chars().all(|c| c.is_ascii_lowercase() || c == '_')
    {
        let mut d = std::collections::HashMap::new();
        d.insert(
            "name".to_string(),
            vec!["Role name must be 2-50 lowercase letters/underscores".to_string()],
        );
        return Err(AppError::Validation(d));
    }
    Ok(())
}
