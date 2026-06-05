use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use tracing::error;

// ─── Domain errors ────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum AppError {
    // Auth
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Account is locked")]
    AccountLocked,
    #[error("Account is not verified")]
    AccountNotVerified,
    #[error("Account is inactive")]
    AccountInactive,
    #[error("Email already taken")]
    EmailTaken,
    #[error("Invalid or expired token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("MFA required")]
    MfaRequired,
    #[error("Invalid MFA code")]
    InvalidMfaCode,
    #[error("MFA not enabled")]
    MfaNotEnabled,

    // Authorization
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),

    // Resources
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Conflict: {0}")]
    Conflict(String),

    // Validation
    #[error("Validation error")]
    Validation(HashMap<String, Vec<String>>),

    // Rate limiting
    #[error("Too many requests")]
    RateLimited,

    // OAuth
    #[error("OAuth error: {0}")]
    OAuth(String),

    // Internal
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
}

// ─── API Error response ───────────────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, Vec<String>>>,
}

// ─── Convert AppError → HTTP response ────────────────────────────────────────

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message, details) = match &self {
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS",
                self.to_string(),
                None,
            ),
            AppError::AccountLocked => (
                StatusCode::FORBIDDEN,
                "ACCOUNT_LOCKED",
                self.to_string(),
                None,
            ),
            AppError::AccountNotVerified => (
                StatusCode::FORBIDDEN,
                "ACCOUNT_NOT_VERIFIED",
                self.to_string(),
                None,
            ),
            AppError::AccountInactive => (
                StatusCode::FORBIDDEN,
                "ACCOUNT_INACTIVE",
                self.to_string(),
                None,
            ),
            AppError::EmailTaken => (StatusCode::CONFLICT, "EMAIL_TAKEN", self.to_string(), None),
            AppError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "TOKEN_INVALID",
                self.to_string(),
                None,
            ),
            AppError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                "TOKEN_EXPIRED",
                self.to_string(),
                None,
            ),
            AppError::MfaRequired => (
                StatusCode::UNAUTHORIZED,
                "MFA_REQUIRED",
                self.to_string(),
                None,
            ),
            AppError::InvalidMfaCode => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "INVALID_MFA_CODE",
                self.to_string(),
                None,
            ),
            AppError::MfaNotEnabled => (
                StatusCode::BAD_REQUEST,
                "MFA_NOT_ENABLED",
                self.to_string(),
                None,
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                self.to_string(),
                None,
            ),
            AppError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", self.to_string(), None),
            AppError::InsufficientPermissions(_) => (
                StatusCode::FORBIDDEN,
                "INSUFFICIENT_PERMISSIONS",
                self.to_string(),
                None,
            ),
            AppError::NotFound(r) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                format!("{} not found", r),
                None,
            ),
            AppError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", self.to_string(), None),
            AppError::Validation(details) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "VALIDATION_ERROR",
                "Validation failed".to_string(),
                Some(details.clone()),
            ),
            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMITED",
                self.to_string(),
                None,
            ),
            AppError::OAuth(msg) => (StatusCode::BAD_GATEWAY, "OAUTH_ERROR", msg.clone(), None),
            AppError::Database(e) => {
                error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".to_string(),
                    None,
                )
            }
            AppError::Redis(e) => {
                error!("Redis error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".to_string(),
                    None,
                )
            }
            AppError::Internal(e) => {
                error!("Internal error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR",
                    "Internal server error".to_string(),
                    None,
                )
            }
        };

        (
            status,
            Json(ApiError {
                code: code.to_string(),
                message,
                details,
            }),
        )
            .into_response()
    }
}

// ─── Helper type alias ────────────────────────────────────────────────────────

pub type AppResult<T> = Result<T, AppError>;
