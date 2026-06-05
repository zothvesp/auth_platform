use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{LoginHistory, User},
    repositories::base::BaseRepository,
};

pub struct UserRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> UserRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BaseRepository for UserRepository<'_> {
    type Model = User;

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<User>> {
        Ok(
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    async fn find_all(&self) -> AppResult<Vec<User>> {
        Ok(
            sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
                .fetch_all(self.pool)
                .await?,
        )
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

impl UserRepository<'_> {
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        Ok(
            sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
                .bind(email.to_lowercase())
                .fetch_optional(self.pool)
                .await?,
        )
    }

    pub async fn email_exists(&self, email: &str) -> AppResult<bool> {
        let row: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(email.to_lowercase())
            .fetch_one(self.pool)
            .await?;
        Ok(row.0)
    }

    pub async fn find_paginated(
        &self,
        page: i64,
        page_size: i64,
        status_filter: Option<&str>,
    ) -> AppResult<(Vec<User>, i64)> {
        let offset = (page - 1) * page_size;
        let (users, total) =
            if let Some(status) = status_filter {
                let users = sqlx::query_as::<_, User>(
                "SELECT * FROM users WHERE status = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3")
                .bind(status).bind(page_size).bind(offset).fetch_all(self.pool).await?;
                let (total,): (i64,) =
                    sqlx::query_as("SELECT COUNT(*) FROM users WHERE status = $1")
                        .bind(status)
                        .fetch_one(self.pool)
                        .await?;
                (users, total)
            } else {
                let users = sqlx::query_as::<_, User>(
                    "SELECT * FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2",
                )
                .bind(page_size)
                .bind(offset)
                .fetch_all(self.pool)
                .await?;
                let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
                    .fetch_one(self.pool)
                    .await?;
                (users, total)
            };
        Ok((users, total))
    }

    pub async fn create<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        id: Uuid,
        email: &str,
        display_name: &str,
        password_hash: Option<&str>,
        auth_method: &str,
    ) -> AppResult<User> {
        let now = Utc::now();
        Ok(sqlx::query_as::<_, User>(
            "INSERT INTO users (id, email, display_name, password_hash, email_verified, status, mfa_enabled, auth_method, created_at, updated_at)
             VALUES ($1, $2, $3, $4, false, 'active', false, $5, $6, $6) RETURNING *")
            .bind(id).bind(email.to_lowercase()).bind(display_name).bind(password_hash).bind(auth_method).bind(now)
            .fetch_one(exec).await?)
    }

    pub async fn update_last_login<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("UPDATE users SET last_login_at = $1 WHERE id = $2")
            .bind(Utc::now())
            .bind(id)
            .execute(exec)
            .await?;
        Ok(())
    }

    pub async fn set_email_verified<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("UPDATE users SET email_verified = true, updated_at = $1 WHERE id = $2")
            .bind(Utc::now())
            .bind(id)
            .execute(exec)
            .await?;
        Ok(())
    }

    pub async fn update_password<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        id: Uuid,
        password_hash: &str,
    ) -> AppResult<()> {
        sqlx::query("UPDATE users SET password_hash = $1, updated_at = $2 WHERE id = $3")
            .bind(password_hash)
            .bind(Utc::now())
            .bind(id)
            .execute(exec)
            .await?;
        Ok(())
    }

    pub async fn update_profile<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        id: Uuid,
        display_name: &str,
        avatar_url: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            "UPDATE users SET display_name = $1, avatar_url = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(display_name)
        .bind(avatar_url)
        .bind(Utc::now())
        .bind(id)
        .execute(exec)
        .await?;
        Ok(())
    }

    pub async fn set_status<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        id: Uuid,
        status: &str,
    ) -> AppResult<()> {
        sqlx::query("UPDATE users SET status = $1, updated_at = $2 WHERE id = $3")
            .bind(status)
            .bind(Utc::now())
            .bind(id)
            .execute(exec)
            .await?;
        Ok(())
    }

    pub async fn set_mfa<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        id: Uuid,
        enabled: bool,
        secret: Option<&str>,
    ) -> AppResult<()> {
        sqlx::query(
            "UPDATE users SET mfa_enabled = $1, mfa_secret = $2, updated_at = $3 WHERE id = $4",
        )
        .bind(enabled)
        .bind(secret)
        .bind(Utc::now())
        .bind(id)
        .execute(exec)
        .await?;
        Ok(())
    }

    pub async fn assign_role<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
        role_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(user_id)
        .bind(role_id)
        .execute(exec)
        .await?;
        Ok(())
    }

    pub async fn remove_role<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
        role_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1 AND role_id = $2")
            .bind(user_id)
            .bind(role_id)
            .execute(exec)
            .await?;
        Ok(())
    }

    pub async fn record_login<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
        ip: &str,
        user_agent: &str,
        success: bool,
        auth_method: &str,
    ) -> AppResult<()> {
        sqlx::query("INSERT INTO login_history (id, user_id, ip_address, user_agent, success, auth_method, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)")
            .bind(Uuid::new_v4()).bind(user_id).bind(ip).bind(user_agent).bind(success).bind(auth_method).bind(Utc::now())
            .execute(exec).await?;
        Ok(())
    }

    pub async fn login_history(&self, user_id: Uuid, limit: i64) -> AppResult<Vec<LoginHistory>> {
        Ok(sqlx::query_as::<_, LoginHistory>(
            "SELECT * FROM login_history WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2",
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(self.pool)
        .await?)
    }
}
