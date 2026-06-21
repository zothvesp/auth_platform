use crate::impl_entity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_email: Option<String>,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub ip_address: String,
    pub user_agent: String,
    pub success: bool,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl_entity!(AuditLog);

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LoginHistory {
    pub id: Uuid,
    pub user_id: Uuid,
    pub ip_address: String,
    pub user_agent: String,
    pub location: Option<String>,
    pub success: bool,
    pub auth_method: String,
    pub created_at: DateTime<Utc>,
}

impl_entity!(LoginHistory);
