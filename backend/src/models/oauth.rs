use chrono::{DateTime, Utc};

use crate::impl_entity;
use sqlx::FromRow;
use uuid::Uuid;

// ─── OAuth Account ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, FromRow)]
pub struct OAuthAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: String,
    pub provider_user_id: String,
    pub provider_email: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl_entity!(OAuthAccount);
