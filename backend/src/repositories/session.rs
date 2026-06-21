use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::Session,
};

pub struct SessionRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> SessionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn create<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        id: Uuid,
        user_id: Uuid,
        ip_address: &str,
        user_agent: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO sessions (id, user_id, ip_address, user_agent, expires_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO UPDATE SET
               ip_address = EXCLUDED.ip_address,
               user_agent = EXCLUDED.user_agent,
               expires_at = EXCLUDED.expires_at",
        )
        .bind(id)
        .bind(user_id)
        .bind(ip_address)
        .bind(user_agent)
        .bind(expires_at)
        .bind(Utc::now())
        .execute(exec)
        .await?;
        Ok(())
    }

    pub async fn find_active_by_user(&self, user_id: Uuid) -> AppResult<Vec<Session>> {
        Ok(sqlx::query_as::<_, Session>(
            "SELECT * FROM sessions
             WHERE user_id = $1 AND expires_at > $2
             ORDER BY created_at DESC",
        )
        .bind(user_id)
        .bind(Utc::now())
        .fetch_all(self.pool)
        .await?)
    }

    pub async fn delete_by_id<'e, E: sqlx::PgExecutor<'e>>(exec: E, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(exec)
            .await?;
        Ok(())
    }

    pub async fn delete_by_user<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user_id)
            .execute(exec)
            .await?;
        Ok(())
    }
}
