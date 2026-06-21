use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppResult,
    middleware::auth::AuthUser,
    services::saml,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // SAML login initiation (public)
        .route("/saml/:provider/login", get(saml_login))
        // SAML callback (public)
        .route("/saml/callback", post(saml_callback))
        // SAML IdP metadata (public)
        .route("/saml/:provider/metadata", get(saml_metadata))
        // Admin: manage SAML providers
        .route("/saml/providers", get(list_providers).post(create_provider))
        .route(
            "/saml/providers/:id",
            delete(delete_provider),
        )
        .route("/saml/providers/:id/toggle", post(toggle_provider))
}

#[derive(Deserialize)]
pub struct SamlLoginParams {
    pub provider: String,
}

#[derive(Deserialize)]
pub struct SamlCallbackBody {
    pub saml_response: String,
    pub relay_state: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub display_name: String,
    pub entity_id: String,
    pub sso_url: String,
    pub certificate: String,
}

#[derive(Deserialize)]
pub struct ToggleProviderRequest {
    pub enabled: bool,
}

pub async fn saml_login(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> AppResult<impl IntoResponse> {
    let result = saml::initiate_saml_login(&state, &provider).await?;
    Ok(Json(result))
}

pub async fn saml_metadata(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> AppResult<Response> {
    let xml = saml::generate_metadata(&state, &provider).await?;
    Ok(([("content-type", "application/xml")], xml).into_response())
}

pub async fn saml_callback(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<SamlCallbackBody>,
) -> AppResult<impl IntoResponse> {
    let ip = crate::utils::extract_ip(&headers);
    let user_agent = crate::utils::extract_ua(&headers);
    let (user, access_token, refresh_token) =
        saml::handle_saml_callback(&state, &body.saml_response, &ip, &user_agent).await?;
    let expires_in = crate::services::config::jwt_access_expiry_secs(&state).await;
    Ok(super::auth::auth_response(user, access_token, refresh_token, expires_in, &state).await)
}

pub async fn list_providers(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("settings:manage")?;
    Ok(Json(saml::list_providers(&state).await?))
}

pub async fn create_provider(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateProviderRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("settings:manage")?;
    let provider = saml::create_provider(
        &state,
        &req.name,
        &req.display_name,
        &req.entity_id,
        &req.sso_url,
        &req.certificate,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(provider)))
}

pub async fn delete_provider(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("settings:manage")?;
    saml::delete_provider(&state, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn toggle_provider(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<ToggleProviderRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("settings:manage")?;
    saml::toggle_provider(&state, id, body.enabled).await?;
    Ok(StatusCode::NO_CONTENT)
}
