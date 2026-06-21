use crate::{error::AppResult, models::AuthorizationCode};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct AuthorizationCodeRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> AuthorizationCodeRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        code_hash: &str,
        client_id: &str,
        user_id: Uuid,
        redirect_uri: &str,
        scope: &str,
        code_challenge: Option<&str>,
        code_challenge_method: Option<&str>,
        expires_at: chrono::DateTime<Utc>,
    ) -> AppResult<AuthorizationCode> {
        Ok(sqlx::query_as::<_, AuthorizationCode>(
            "INSERT INTO authorization_codes (id, code_hash, client_id, user_id, redirect_uri, scope, code_challenge, code_challenge_method, expires_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
             RETURNING *",
        )
        .bind(Uuid::new_v4())
        .bind(code_hash)
        .bind(client_id)
        .bind(user_id)
        .bind(redirect_uri)
        .bind(scope)
        .bind(code_challenge)
        .bind(code_challenge_method)
        .bind(expires_at)
        .fetch_one(self.pool)
        .await?)
    }

    pub async fn find_by_hash(&self, code_hash: &str) -> AppResult<Option<AuthorizationCode>> {
        Ok(
            sqlx::query_as::<_, AuthorizationCode>(
                "SELECT * FROM authorization_codes WHERE code_hash = $1",
            )
            .bind(code_hash)
            .fetch_optional(self.pool)
            .await?,
        )
    }

    pub async fn mark_used(&self, code_hash: &str) -> AppResult<()> {
        sqlx::query("UPDATE authorization_codes SET used_at = NOW() WHERE code_hash = $1")
            .bind(code_hash)
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_expired(&self) -> AppResult<()> {
        sqlx::query("DELETE FROM authorization_codes WHERE expires_at < NOW() OR used_at IS NOT NULL")
            .execute(self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_by_user<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM authorization_codes WHERE user_id = $1")
            .bind(user_id)
            .execute(exec)
            .await?;
        Ok(())
    }
}
