use crate::{error::AppResult, models::OAuthAccount, repositories::base::BaseRepository};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

pub struct OAuthRepository<'a> {
    pool: &'a PgPool,
}
impl<'a> OAuthRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BaseRepository for OAuthRepository<'_> {
    type Model = OAuthAccount;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<OAuthAccount>> {
        Ok(
            sqlx::query_as::<_, OAuthAccount>("SELECT * FROM oauth_accounts WHERE id = $1")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }
    async fn find_all(&self) -> AppResult<Vec<OAuthAccount>> {
        Ok(
            sqlx::query_as::<_, OAuthAccount>("SELECT * FROM oauth_accounts")
                .fetch_all(self.pool)
                .await?,
        )
    }
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM oauth_accounts WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

impl OAuthRepository<'_> {
    pub async fn find_by_provider(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> AppResult<Option<OAuthAccount>> {
        Ok(sqlx::query_as::<_, OAuthAccount>(
            "SELECT * FROM oauth_accounts WHERE provider = $1 AND provider_user_id = $2",
        )
        .bind(provider)
        .bind(provider_user_id)
        .fetch_optional(self.pool)
        .await?)
    }
    pub async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<OAuthAccount>> {
        Ok(sqlx::query_as::<_, OAuthAccount>(
            "SELECT * FROM oauth_accounts WHERE user_id = $1 ORDER BY provider",
        )
        .bind(user_id)
        .fetch_all(self.pool)
        .await?)
    }
    pub async fn upsert<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
        provider: &str,
        provider_user_id: &str,
        provider_email: Option<&str>,
        access_token: Option<&str>,
        refresh_token: Option<&str>,
        token_expires_at: Option<DateTime<Utc>>,
    ) -> AppResult<OAuthAccount> {
        let now = Utc::now();
        Ok(sqlx::query_as::<_, OAuthAccount>(
            "INSERT INTO oauth_accounts (id, user_id, provider, provider_user_id, provider_email, access_token, refresh_token, token_expires_at, created_at, updated_at)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$9)
             ON CONFLICT (provider, provider_user_id) DO UPDATE SET
               access_token = EXCLUDED.access_token, refresh_token = EXCLUDED.refresh_token,
               token_expires_at = EXCLUDED.token_expires_at, updated_at = EXCLUDED.updated_at
             RETURNING *")
            .bind(Uuid::new_v4()).bind(user_id).bind(provider).bind(provider_user_id).bind(provider_email).bind(access_token).bind(refresh_token).bind(token_expires_at).bind(now)
            .fetch_one(exec).await?)
    }
}
