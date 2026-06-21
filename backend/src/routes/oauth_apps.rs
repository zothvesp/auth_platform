use axum::{
    extract::{Json, Path, State},
    http::{header::HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
    services::{audit, oauth_apps},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/oauth-apps",
            get(list_apps).post(create_app),
        )
        .route(
            "/oauth-apps/:id",
            get(get_app).put(update_app).delete(delete_app),
        )
}

pub async fn list_apps(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("oauth:read")?;
    let apps = oauth_apps::list_apps(&state).await?;
    Ok(Json(apps))
}

pub async fn create_app(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(input): Json<oauth_apps::CreateOAuthAppInput>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("oauth:create")?;
    input.validate().map_err(|errors| {
        let details = errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                (
                    field.to_string(),
                    errors
                        .iter()
                        .map(|error| error.message.clone().unwrap_or_default().to_string())
                        .collect(),
                )
            })
            .collect();
        AppError::Validation(details)
    })?;

    let result = oauth_apps::create_app(&state, &input).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "oauth_app.create",
        "oauth_app",
        Some(&result.id.to_string()),
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        Some(serde_json::json!({ "client_id": result.client_id, "name": result.name })),
    )
    .await;

    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn get_app(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("oauth:read")?;
    let app = oauth_apps::get_app(&state, id).await?;
    Ok(Json(app))
}

pub async fn update_app(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<oauth_apps::UpdateOAuthAppInput>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("oauth:update")?;
    input.validate().map_err(|errors| {
        let details = errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                (
                    field.to_string(),
                    errors
                        .iter()
                        .map(|error| error.message.clone().unwrap_or_default().to_string())
                        .collect(),
                )
            })
            .collect();
        AppError::Validation(details)
    })?;

    let result = oauth_apps::update_app(&state, id, &input).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "oauth_app.update",
        "oauth_app",
        Some(&id.to_string()),
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        Some(serde_json::json!({ "name": result.name })),
    )
    .await;

    Ok(Json(result))
}

pub async fn delete_app(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("oauth:delete")?;
    oauth_apps::delete_app(&state, id).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "oauth_app.delete",
        "oauth_app",
        Some(&id.to_string()),
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        None,
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}
