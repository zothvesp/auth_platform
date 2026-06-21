use crate::impl_entity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OAuthApp {
    pub id: Uuid,
    pub client_id: String,
    pub client_secret_hash: String,
    pub name: String,
    pub description: Option<String>,
    pub redirect_uris: Vec<String>,
    pub allowed_grants: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub pkce_required: bool,
    pub logo_url: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl_entity!(OAuthApp);
