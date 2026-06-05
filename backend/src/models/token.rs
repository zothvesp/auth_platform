use crate::impl_entity;
use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

/// Long-lived token used to obtain new access tokens.
/// Implements rotation: used_at is set rather than deleted to detect reuse.
#[derive(Debug, Clone, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub family: Uuid, // rotation family — reuse of any token revokes the whole family
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl_entity!(RefreshToken);

/// Single-use token sent to verify a user's email address.
#[derive(Debug, Clone, FromRow)]
pub struct EmailVerificationToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl_entity!(EmailVerificationToken);

/// Single-use HMAC token for password reset flows.
#[derive(Debug, Clone, FromRow)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl_entity!(PasswordResetToken);
