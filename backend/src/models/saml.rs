use crate::impl_entity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SamlProvider {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub entity_id: String,
    pub sso_url: String,
    pub certificate: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl_entity!(SamlProvider);
