use crate::{error::AppResult, models::AuditLog, repositories::base::BaseRepository};
use chrono::Utc;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

pub struct AuditRepository<'a> {
    pool: &'a PgPool,
}
impl<'a> AuditRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BaseRepository for AuditRepository<'_> {
    type Model = AuditLog;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<AuditLog>> {
        Ok(
            sqlx::query_as::<_, AuditLog>("SELECT * FROM audit_logs WHERE id = $1")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }
    async fn find_all(&self) -> AppResult<Vec<AuditLog>> {
        Ok(sqlx::query_as::<_, AuditLog>(
            "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT 1000",
        )
        .fetch_all(self.pool)
        .await?)
    }
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM audit_logs WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

impl AuditRepository<'_> {
    #[allow(clippy::too_many_arguments)]
    pub async fn create<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Option<Uuid>,
        user_email: Option<&str>,
        action: &str,
        resource: &str,
        resource_id: Option<&str>,
        ip: &str,
        user_agent: &str,
        success: bool,
        details: Option<&Value>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO audit_logs (id, user_id, user_email, action, resource, resource_id, ip_address, user_agent, success, details, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)")
            .bind(Uuid::new_v4()).bind(user_id).bind(user_email).bind(action).bind(resource).bind(resource_id).bind(ip).bind(user_agent).bind(success).bind(details).bind(Utc::now())
            .execute(exec).await?;
        Ok(())
    }

    pub async fn find_paginated(
        &self,
        page: i64,
        page_size: i64,
        user_id_filter: Option<Uuid>,
    ) -> AppResult<(Vec<AuditLog>, i64)> {
        let offset = (page - 1) * page_size;
        let (logs, total) = if let Some(uid) = user_id_filter {
            let logs = sqlx::query_as::<_, AuditLog>(
                "SELECT * FROM audit_logs WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
                .bind(uid).bind(page_size).bind(offset).fetch_all(self.pool).await?;
            let (total,): (i64,) =
                sqlx::query_as("SELECT COUNT(*) FROM audit_logs WHERE user_id = $1")
                    .bind(uid)
                    .fetch_one(self.pool)
                    .await?;
            (logs, total)
        } else {
            let logs = sqlx::query_as::<_, AuditLog>(
                "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            )
            .bind(page_size)
            .bind(offset)
            .fetch_all(self.pool)
            .await?;
            let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM audit_logs")
                .fetch_one(self.pool)
                .await?;
            (logs, total)
        };
        Ok((logs, total))
    }
}

pub fn parse_user_agent(ua: &str) -> Value {
    let browser = if ua.contains("Edg/") {
        "Edge"
    } else if ua.contains("OPR/") || ua.contains("Opera") {
        "Opera"
    } else if ua.contains("Chrome") && !ua.contains("Edg/") {
        "Chrome"
    } else if ua.contains("Firefox") {
        "Firefox"
    } else if ua.contains("Safari") && !ua.contains("Chrome") {
        "Safari"
    } else {
        "Unknown"
    };

    let os = if ua.contains("Windows") {
        "Windows"
    } else if ua.contains("Mac OS") || ua.contains("Macintosh") {
        "macOS"
    } else if ua.contains("Linux") && !ua.contains("Android") {
        "Linux"
    } else if ua.contains("Android") {
        "Android"
    } else if ua.contains("iPhone") || ua.contains("iPad") {
        "iOS"
    } else {
        "Unknown"
    };

    json!({ "browser": browser, "os": os })
}
