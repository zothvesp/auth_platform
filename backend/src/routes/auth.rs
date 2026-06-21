//! Auth route handlers — thin HTTP layer only.
//! No business logic, no SQL, no crypto. Delegates everything to services.

use axum::{
    extract::{Json, Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
    services,
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/refresh", post(refresh))
        .route("/verify/:token", get(verify_email))
        .route("/resend-verification", post(resend_verification))
        .route("/forgot-password", post(forgot_password))
        .route("/reset-password", post(reset_password))
        .route("/check-permission", post(check_permission))
        .route("/check-permissions", post(batch_check_permissions))
        .route("/mfa/setup", post(mfa_setup))
        .route("/mfa/verify", post(mfa_verify))
        .route("/mfa/disable", post(mfa_disable))
}

// ─── Request DTOs ─────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    #[validate(length(min = 2, max = 50))]
    pub display_name: String,
    #[validate(length(min = 1, message = "Password is required"))]
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub remember_me: Option<bool>,
    pub mfa_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResetPasswordRequest {
    pub token: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckPermissionRequest {
    pub permission: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchCheckPermissionsRequest {
    pub permissions: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MfaCodeRequest {
    pub code: String,
}

// ─── Response DTOs ────────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct AuthResponse {
    pub user: services::auth::UserDto,
    pub tokens: TokenDto,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenDto {
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
    pub expires_at: i64,
}

// ─── Handlers ─────────────────────────────────────────────────────────────────

pub async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> AppResult<impl IntoResponse> {
    // Basic structural validation first (not policy — policy is checked in service)
    req.validate().map_err(|e| {
        let details = e
            .field_errors()
            .into_iter()
            .map(|(k, v)| {
                (
                    k.to_string(),
                    v.iter()
                        .map(|e| e.message.clone().unwrap_or_default().to_string())
                        .collect(),
                )
            })
            .collect();
        AppError::Validation(details)
    })?;

    // Block registration if disabled in config
    if !services::config::allow_registration(&state).await {
        return Err(AppError::Forbidden);
    }

    let ip = extract_ip(&headers);
    let ua = extract_ua(&headers);
    let (user, access_token, refresh_token) = services::auth::register(
        &state,
        &req.email,
        &req.display_name,
        &req.password,
        &ip,
        &ua,
    )
    .await?;

    let expires_in = services::config::jwt_access_expiry_secs(&state).await;
    Ok(auth_response(user, access_token, refresh_token, expires_in, &state).await)
}

pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> AppResult<impl IntoResponse> {
    let ip = extract_ip(&headers);
    let ua = extract_ua(&headers);
    let (user, access_token, refresh_token) = services::auth::login(
        &state,
        &req.email,
        &req.password,
        req.mfa_code.as_deref(),
        &ip,
        &ua,
        req.remember_me.unwrap_or(false),
    )
    .await?;

    services::audit::record(
        &state,
        Some(user.id),
        Some(&user.email),
        "auth.login",
        "auth",
        None,
        &ip,
        &ua,
        true,
        None,
    )
    .await;

    let expires_in = services::config::jwt_access_expiry_secs(&state).await;
    Ok(auth_response(user, access_token, refresh_token, expires_in, &state).await)
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    auth: Option<AuthUser>,
) -> AppResult<impl IntoResponse> {
    let ip = extract_ip(&headers);
    let ua = extract_ua(&headers);

    // Extract refresh token from cookie header
    if let Some(token) = extract_refresh_cookie(&headers) {
        services::auth::logout(&state, &token).await.ok();
    }

    if let Some(ref user) = auth {
        services::audit::record(
            &state,
            Some(user.user_id),
            Some(&user.email),
            "auth.logout",
            "auth",
            None,
            &ip,
            &ua,
            true,
            None,
        )
        .await;
    }

    Ok((
        StatusCode::NO_CONTENT,
        [(header::SET_COOKIE, clear_cookie(&state))],
    ))
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<impl IntoResponse> {
    let token = extract_refresh_cookie(&headers).ok_or(AppError::Unauthorized)?;
    let ip = extract_ip(&headers);
    let ua = extract_ua(&headers);
    let (access_token, new_refresh, expires_in) =
        services::auth::refresh_token(&state, &token, &ip, &ua).await?;

    let cookie = make_refresh_cookie(&new_refresh, &state).await;
    let expires_at = chrono::Utc::now().timestamp() + expires_in as i64;
    Ok((
        [(header::SET_COOKIE, cookie)],
        Json(TokenDto {
            access_token,
            token_type: "Bearer",
            expires_in,
            expires_at,
        }),
    ))
}

pub async fn verify_email(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> AppResult<impl IntoResponse> {
    services::auth::verify_email(&state, &token).await?;
    Ok(Json(
        serde_json::json!({ "message": "Email verified successfully" }),
    ))
}

pub async fn resend_verification(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    services::email::send_verification(&state, auth.user_id).await?;
    Ok(Json(
        serde_json::json!({ "message": "Verification email sent" }),
    ))
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> AppResult<impl IntoResponse> {
    // Always 200 — prevents email enumeration
    services::auth::forgot_password(&state, &req.email)
        .await
        .ok();
    Ok(Json(
        serde_json::json!({ "message": "If that email exists, a reset link has been sent" }),
    ))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> AppResult<impl IntoResponse> {
    services::auth::reset_password(&state, &req.token, &req.password).await?;
    Ok(Json(
        serde_json::json!({ "message": "Password updated successfully" }),
    ))
}

pub async fn check_permission(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CheckPermissionRequest>,
) -> AppResult<impl IntoResponse> {
    let allowed =
        services::rbac::user_has_permission(&state, auth.user_id, &req.permission).await?;
    Ok(Json(serde_json::json!({ "allowed": allowed })))
}

pub async fn batch_check_permissions(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<BatchCheckPermissionsRequest>,
) -> AppResult<impl IntoResponse> {
    let results =
        services::rbac::batch_check_permissions(&state, auth.user_id, &req.permissions).await?;
    Ok(Json(results))
}

pub async fn mfa_setup(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<impl IntoResponse> {
    let (qr_code, secret) = services::mfa::setup(&state, auth.user_id).await?;
    Ok(Json(
        serde_json::json!({ "qr_code": qr_code, "secret": secret }),
    ))
}

pub async fn mfa_verify(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<MfaCodeRequest>,
) -> AppResult<impl IntoResponse> {
    let backup_codes = services::mfa::enable(&state, auth.user_id, &req.code).await?;
    Ok(Json(serde_json::json!({ "backup_codes": backup_codes })))
}

pub async fn mfa_disable(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<MfaCodeRequest>,
) -> AppResult<impl IntoResponse> {
    services::mfa::disable(&state, auth.user_id, &req.code).await?;
    Ok(Json(serde_json::json!({ "message": "MFA disabled" })))
}

// ─── Private helpers ──────────────────────────────────────────────────────────

fn extract_ip(headers: &HeaderMap) -> String {
    crate::utils::extract_ip(headers)
}

fn extract_ua(headers: &HeaderMap) -> String {
    crate::utils::extract_ua(headers)
}

fn extract_refresh_cookie(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .find(|s| s.trim().starts_with("refresh_token="))
        .map(|s| s.trim().trim_start_matches("refresh_token=").to_string())
}

async fn make_refresh_cookie(token: &str, state: &AppState) -> String {
    let secure = if services::config::cookie_secure(state).await {
        "; Secure"
    } else {
        ""
    };
    let same_site = services::config::same_site_policy(state).await;
    let max_age = services::config::jwt_refresh_expiry_secs(state).await;
    format!(
        "refresh_token={}; HttpOnly{}; SameSite={}; Path=/api/v1/auth; Max-Age={}; Domain={}",
        token, secure, same_site, max_age, state.config.cookie_domain
    )
}

fn clear_cookie(state: &AppState) -> String {
    format!(
        "refresh_token=; HttpOnly; SameSite=Strict; Path=/api/v1/auth; Max-Age=0; Domain={}",
        state.config.cookie_domain
    )
}

pub(crate) async fn auth_response(
    user: services::auth::UserDto,
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    state: &AppState,
) -> impl IntoResponse {
    let cookie = make_refresh_cookie(&refresh_token, state).await;
    let expires_at = chrono::Utc::now().timestamp() + expires_in as i64;
    (
        [(header::SET_COOKIE, cookie)],
        Json(AuthResponse {
            user,
            tokens: TokenDto {
                access_token,
                token_type: "Bearer",
                expires_in,
                expires_at,
            },
        }),
    )
}
