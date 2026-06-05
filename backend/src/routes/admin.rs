use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
    repositories::{base::BaseRepository, AuditRepository, ConfigRepository, UserRepository},
    services,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // User management
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user).put(update_user))
        .route("/users/:id/deactivate", patch(deactivate_user))
        .route("/users/:id/reactivate", patch(reactivate_user))
        .route("/users/:id", delete(delete_user))
        .route("/users/:id/roles", post(assign_role).delete(remove_role))
        // Audit
        .route("/audit-logs", get(list_audit_logs))
}

#[derive(Deserialize)]
pub struct UserListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub status: Option<String>,
}

#[derive(Serialize)]
pub struct PaginatedUsers {
    pub data: Vec<services::auth::UserDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Deserialize)]
pub struct AuditQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub user_id: Option<Uuid>,
}

pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<UserListQuery>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:read")?;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 100);
    let (users, total) = UserRepository::new(&state.db.pool)
        .find_paginated(page, page_size, q.status.as_deref())
        .await?;

    // Build DTOs for each user
    let mut dtos = Vec::with_capacity(users.len());
    for user in users {
        dtos.push(services::auth::build_user_dto(&state, user.id).await?);
    }

    let total_pages = (total as f64 / page_size as f64).ceil() as i64;
    Ok(Json(PaginatedUsers {
        data: dtos,
        total,
        page,
        page_size,
        total_pages,
    }))
}

pub async fn get_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:read")?;
    Ok(Json(services::auth::build_user_dto(&state, id).await?))
}

pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:update")?;
    if let Some(name) = req.display_name.as_deref() {
        validate_display_name(&state, name).await?;
        let avatar = req
            .avatar_url
            .as_deref()
            .filter(|value| !value.trim().is_empty());
        UserRepository::update_profile(&state.db.pool, id, name, avatar).await?;
    }
    Ok(Json(services::auth::build_user_dto(&state, id).await?))
}

pub async fn deactivate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:manage")?;
    UserRepository::set_status(&state.db.pool, id, "inactive").await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn reactivate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:manage")?;
    UserRepository::set_status(&state.db.pool, id, "active").await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:delete")?;
    UserRepository::new(&state.db.pool).delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn assign_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AssignRoleRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:manage")?;
    UserRepository::assign_role(&state.db.pool, user_id, req.role_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_role(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AssignRoleRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:manage")?;
    UserRepository::remove_role(&state.db.pool, user_id, req.role_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_audit_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<AuditQuery>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("audit:read")?;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(50).clamp(1, 200);
    let (logs, total) = AuditRepository::new(&state.db.pool)
        .find_paginated(page, page_size, q.user_id)
        .await?;
    let total_pages = (total as f64 / page_size as f64).ceil() as i64;
    Ok(Json(
        serde_json::json!({ "data": logs, "total": total, "page": page, "page_size": page_size, "total_pages": total_pages }),
    ))
}

async fn validate_display_name(state: &AppState, name: &str) -> AppResult<()> {
    let rules = ConfigRepository::new(&state.db.pool)
        .public_config()
        .await?
        .validation_rules;
    let trimmed_len = name.trim().chars().count() as u32;
    let mut errors = Vec::new();

    if trimmed_len < rules.display_name_min {
        errors.push(format!(
            "Display name must be at least {} characters",
            rules.display_name_min
        ));
    }
    if trimmed_len > rules.display_name_max {
        errors.push(format!(
            "Display name must be {} characters or less",
            rules.display_name_max
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(std::collections::HashMap::from([(
            "displayName".to_string(),
            errors,
        )])))
    }
}
