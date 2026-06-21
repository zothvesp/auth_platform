use axum::{
    extract::{Json, Path, State},
    http::{header, HeaderMap},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::{error::AppResult, routes::auth, services, state::AppState};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:provider", get(oauth_redirect))
        .route("/:provider/callback", post(oauth_callback))
}

#[derive(Deserialize)]
pub struct CallbackRequest {
    pub code: String,
    pub state: String,
}

pub async fn oauth_redirect(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> AppResult<impl IntoResponse> {
    Ok(Json(
        services::oauth::authorization_url(&state, &provider).await?,
    ))
}

pub async fn oauth_callback(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    headers: HeaderMap,
    Json(req): Json<CallbackRequest>,
) -> AppResult<impl IntoResponse> {
    let ip = extract_ip(&headers);
    let user_agent = headers
        .get(header::USER_AGENT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unknown");
    let (user, access_token, refresh_token) =
        services::oauth::callback(&state, &provider, &req.code, &req.state, &ip, user_agent)
            .await?;
    let expires_in = services::config::jwt_access_expiry_secs(&state).await;
    Ok(auth::auth_response(user, access_token, refresh_token, expires_in, &state).await)
}

fn extract_ip(headers: &HeaderMap) -> String {
    crate::utils::extract_ip(headers)
}
