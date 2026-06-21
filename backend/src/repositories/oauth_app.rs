use crate::{error::AppResult, models::OAuthApp, repositories::base::BaseRepository};
use sqlx::PgPool;
use uuid::Uuid;

pub struct OAuthAppRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> OAuthAppRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BaseRepository for OAuthAppRepository<'_> {
    type Model = OAuthApp;

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<OAuthApp>> {
        Ok(
            sqlx::query_as::<_, OAuthApp>("SELECT * FROM oauth_apps WHERE id = $1")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    async fn find_all(&self) -> AppResult<Vec<OAuthApp>> {
        Ok(
            sqlx::query_as::<_, OAuthApp>("SELECT * FROM oauth_apps ORDER BY name")
                .fetch_all(self.pool)
                .await?,
        )
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM oauth_apps WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

impl OAuthAppRepository<'_> {
    pub async fn find_by_client_id(&self, client_id: &str) -> AppResult<Option<OAuthApp>> {
        Ok(
            sqlx::query_as::<_, OAuthApp>("SELECT * FROM oauth_apps WHERE client_id = $1")
                .bind(client_id)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        client_id: &str,
        client_secret_hash: &str,
        name: &str,
        description: Option<&str>,
        redirect_uris: &[String],
        allowed_grants: &[String],
        allowed_scopes: &[String],
        pkce_required: bool,
        logo_url: Option<&str>,
    ) -> AppResult<OAuthApp> {
        Ok(sqlx::query_as::<_, OAuthApp>(
            "INSERT INTO oauth_apps (id, client_id, client_secret_hash, name, description, redirect_uris, allowed_grants, allowed_scopes, pkce_required, logo_url, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, NOW(), NOW())
             RETURNING *",
        )
        .bind(Uuid::new_v4())
        .bind(client_id)
        .bind(client_secret_hash)
        .bind(name)
        .bind(description)
        .bind(redirect_uris)
        .bind(allowed_grants)
        .bind(allowed_scopes)
        .bind(pkce_required)
        .bind(logo_url)
        .fetch_one(self.pool)
        .await?)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: Uuid,
        name: &str,
        description: Option<&str>,
        redirect_uris: &[String],
        allowed_grants: &[String],
        allowed_scopes: &[String],
        pkce_required: bool,
        logo_url: Option<&str>,
        is_active: bool,
    ) -> AppResult<()> {
        sqlx::query(
            "UPDATE oauth_apps SET name = $1, description = $2, redirect_uris = $3, allowed_grants = $4, allowed_scopes = $5, pkce_required = $6, logo_url = $7, is_active = $8, updated_at = NOW() WHERE id = $9",
        )
        .bind(name)
        .bind(description)
        .bind(redirect_uris)
        .bind(allowed_grants)
        .bind(allowed_scopes)
        .bind(pkce_required)
        .bind(logo_url)
        .bind(is_active)
        .bind(id)
        .execute(self.pool)
        .await?;
        Ok(())
    }
}
