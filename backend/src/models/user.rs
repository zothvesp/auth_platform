use crate::impl_entity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub password_hash: Option<String>,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub status: String, // active | inactive | suspended
    pub mfa_enabled: bool,
    pub mfa_secret: Option<String>,
    pub auth_method: String, // password | google | github | microsoft | saml | oidc
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl_entity!(User);
