use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppResult,
    middleware::auth::AuthUser,
    repositories::{base::BaseRepository, PermissionRepository},
    services,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_permissions).post(create_permission))
        .route("/:id", get(get_permission).delete(delete_permission))
}

#[derive(Deserialize)]
pub struct CreatePermissionRequest {
    pub resource: String,
    pub action: String,
    pub description: String,
}

pub async fn list_permissions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("permissions:read")?;
    Ok(Json(services::rbac::list_permissions(&state).await?))
}

pub async fn get_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("permissions:read")?;
    Ok(Json(
        PermissionRepository::new(&state.db.pool).get(id).await?,
    ))
}

pub async fn create_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreatePermissionRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("permissions:manage")?;
    validate_permission_field("resource", &req.resource)?;
    validate_permission_field("action", &req.action)?;
    let perm =
        PermissionRepository::create(&state.db.pool, &req.resource, &req.action, &req.description)
            .await?;
    Ok((StatusCode::CREATED, Json(perm)))
}

pub async fn delete_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("permissions:manage")?;
    PermissionRepository::new(&state.db.pool).delete(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn validate_permission_field(field: &str, value: &str) -> crate::error::AppResult<()> {
    if value.len() < 2 || !value.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
        let mut d = std::collections::HashMap::new();
        d.insert(
            field.to_string(),
            vec![format!(
                "{} must be 2+ lowercase letters/underscores",
                field
            )],
        );
        return Err(crate::error::AppError::Validation(d));
    }
    Ok(())
}
