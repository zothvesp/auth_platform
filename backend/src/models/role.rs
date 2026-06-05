use crate::impl_entity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_system: bool,
    pub parent_role_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl_entity!(Role);

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub resource: String,
    pub action: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

impl_entity!(Permission);

impl Permission {
    /// Canonical "resource:action" key — the string used in checks.
    pub fn key(&self) -> String {
        format!("{}:{}", self.resource, self.action)
    }
}
