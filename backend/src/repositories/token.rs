use crate::{
    error::AppResult,
    models::{EmailVerificationToken, PasswordResetToken, RefreshToken},
    repositories::base::BaseRepository,
};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

// ─── RefreshTokenRepository ───────────────────────────────────────────────────
pub struct RefreshTokenRepository<'a> {
    pool: &'a PgPool,
}
impl<'a> RefreshTokenRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BaseRepository for RefreshTokenRepository<'_> {
    type Model = RefreshToken;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<RefreshToken>> {
        Ok(
            sqlx::query_as::<_, RefreshToken>("SELECT * FROM refresh_tokens WHERE id = $1")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }
    async fn find_all(&self) -> AppResult<Vec<RefreshToken>> {
        Ok(
            sqlx::query_as::<_, RefreshToken>("SELECT * FROM refresh_tokens")
                .fetch_all(self.pool)
                .await?,
        )
    }
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

impl RefreshTokenRepository<'_> {
    pub async fn find_by_hash(&self, hash: &str) -> AppResult<Option<RefreshToken>> {
        Ok(
            sqlx::query_as::<_, RefreshToken>("SELECT * FROM refresh_tokens WHERE token_hash = $1")
                .bind(hash)
                .fetch_optional(self.pool)
                .await?,
        )
    }
    pub async fn create<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
        token_hash: &str,
        family: Uuid,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query("INSERT INTO refresh_tokens (id, user_id, token_hash, family, expires_at, created_at) VALUES ($1,$2,$3,$4,$5,$6)")
            .bind(Uuid::new_v4()).bind(user_id).bind(token_hash).bind(family).bind(expires_at).bind(Utc::now())
            .execute(exec).await?;
        Ok(())
    }
    pub async fn mark_used<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        token_hash: &str,
    ) -> AppResult<()> {
        sqlx::query("UPDATE refresh_tokens SET used_at = $1 WHERE token_hash = $2")
            .bind(Utc::now())
            .bind(token_hash)
            .execute(exec)
            .await?;
        Ok(())
    }
    pub async fn delete_by_hash<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        token_hash: &str,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE token_hash = $1")
            .bind(token_hash)
            .execute(exec)
            .await?;
        Ok(())
    }
    pub async fn revoke_family<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        family: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE family = $1")
            .bind(family)
            .execute(exec)
            .await?;
        Ok(())
    }
    pub async fn delete_by_user<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM refresh_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(exec)
            .await?;
        Ok(())
    }
}

// ─── EmailTokenRepository ─────────────────────────────────────────────────────
pub struct EmailTokenRepository<'a> {
    pool: &'a PgPool,
}
impl<'a> EmailTokenRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BaseRepository for EmailTokenRepository<'_> {
    type Model = EmailVerificationToken;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<EmailVerificationToken>> {
        Ok(sqlx::query_as::<_, EmailVerificationToken>(
            "SELECT * FROM email_verification_tokens WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?)
    }
    async fn find_all(&self) -> AppResult<Vec<EmailVerificationToken>> {
        Ok(
            sqlx::query_as::<_, EmailVerificationToken>("SELECT * FROM email_verification_tokens")
                .fetch_all(self.pool)
                .await?,
        )
    }
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM email_verification_tokens WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

impl EmailTokenRepository<'_> {
    pub async fn find_by_hash(&self, hash: &str) -> AppResult<Option<EmailVerificationToken>> {
        Ok(sqlx::query_as::<_, EmailVerificationToken>(
            "SELECT * FROM email_verification_tokens WHERE token_hash = $1",
        )
        .bind(hash)
        .fetch_optional(self.pool)
        .await?)
    }
    pub async fn create<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query("INSERT INTO email_verification_tokens (id, user_id, token_hash, expires_at, created_at) VALUES ($1,$2,$3,$4,$5)")
            .bind(Uuid::new_v4()).bind(user_id).bind(token_hash).bind(expires_at).bind(Utc::now())
            .execute(exec).await?;
        Ok(())
    }
    pub async fn delete_by_user<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM email_verification_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(exec)
            .await?;
        Ok(())
    }
}

// ─── PasswordResetTokenRepository ─────────────────────────────────────────────
pub struct PasswordResetTokenRepository<'a> {
    pool: &'a PgPool,
}
impl<'a> PasswordResetTokenRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BaseRepository for PasswordResetTokenRepository<'_> {
    type Model = PasswordResetToken;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<PasswordResetToken>> {
        Ok(sqlx::query_as::<_, PasswordResetToken>(
            "SELECT * FROM password_reset_tokens WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?)
    }
    async fn find_all(&self) -> AppResult<Vec<PasswordResetToken>> {
        Ok(
            sqlx::query_as::<_, PasswordResetToken>("SELECT * FROM password_reset_tokens")
                .fetch_all(self.pool)
                .await?,
        )
    }
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM password_reset_tokens WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

impl PasswordResetTokenRepository<'_> {
    pub async fn find_unused_by_hash(&self, hash: &str) -> AppResult<Option<PasswordResetToken>> {
        Ok(sqlx::query_as::<_, PasswordResetToken>(
            "SELECT * FROM password_reset_tokens WHERE token_hash = $1 AND used_at IS NULL",
        )
        .bind(hash)
        .fetch_optional(self.pool)
        .await?)
    }
    pub async fn create<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query("INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at, created_at) VALUES ($1,$2,$3,$4,$5)")
            .bind(Uuid::new_v4()).bind(user_id).bind(token_hash).bind(expires_at).bind(Utc::now())
            .execute(exec).await?;
        Ok(())
    }
    pub async fn mark_used<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        token_hash: &str,
    ) -> AppResult<()> {
        sqlx::query("UPDATE password_reset_tokens SET used_at = $1 WHERE token_hash = $2")
            .bind(Utc::now())
            .bind(token_hash)
            .execute(exec)
            .await?;
        Ok(())
    }
    pub async fn delete_by_user<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM password_reset_tokens WHERE user_id = $1")
            .bind(user_id)
            .execute(exec)
            .await?;
        Ok(())
    }
}
