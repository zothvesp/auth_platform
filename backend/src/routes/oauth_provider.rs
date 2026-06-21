use axum::{
    extract::{Form, Json, Query, State},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::{
    error::AppResult,
    middleware::auth::AuthUser,
    services::oauth_provider,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/oauth/authorize", get(authorize))
        .route("/oauth/token", post(token))
        .route("/oauth/userinfo", get(userinfo))
        .route("/oauth/jwks", get(jwks))
        .route("/.well-known/openid-configuration", get(oidc_configuration))
        .route("/oauth/introspect", post(introspect))
        .route("/oauth/revoke", post(revoke))
}

pub async fn authorize(
    State(state): State<AppState>,
    Query(params): Query<oauth_provider::AuthorizationRequest>,
) -> AppResult<impl IntoResponse> {
    let response = oauth_provider::authorize(&state, &params).await?;
    Ok(Json(response))
}

pub async fn token(
    State(state): State<AppState>,
    Json(body): Json<oauth_provider::TokenRequest>,
) -> AppResult<impl IntoResponse> {
    let response = oauth_provider::exchange_code(&state, &body).await?;
    Ok(Json(response))
}

pub async fn userinfo(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    let response = oauth_provider::userinfo(&state, auth.user_id, "openid profile email").await?;
    Ok(Json(response))
}

pub async fn jwks(
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let response = oauth_provider::jwks(&state).await?;
    Ok(Json(response))
}

pub async fn oidc_configuration(
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let response = oauth_provider::oidc_configuration(&state);
    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct IntrospectForm {
    token: String,
}

pub async fn introspect(
    State(state): State<AppState>,
    Form(form): Form<IntrospectForm>,
) -> impl IntoResponse {
    let response = oauth_provider::introspect_token(&state, &form.token).await;
    Json(response)
}

#[derive(Deserialize)]
pub struct RevokeForm {
    token: String,
    token_type_hint: Option<String>,
}

pub async fn revoke(
    State(state): State<AppState>,
    Form(form): Form<RevokeForm>,
) -> impl IntoResponse {
    let _ = oauth_provider::revoke_token(&state, &form.token, form.token_type_hint.as_deref()).await;
    axum::http::StatusCode::OK
}
