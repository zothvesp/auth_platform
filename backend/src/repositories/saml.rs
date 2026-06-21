use crate::{error::AppResult, models::SamlProvider, repositories::base::BaseRepository};
use sqlx::PgPool;
use uuid::Uuid;

pub struct SamlRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> SamlRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BaseRepository for SamlRepository<'_> {
    type Model = SamlProvider;

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<SamlProvider>> {
        Ok(
            sqlx::query_as::<_, SamlProvider>("SELECT * FROM saml_providers WHERE id = $1")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    async fn find_all(&self) -> AppResult<Vec<SamlProvider>> {
        Ok(
            sqlx::query_as::<_, SamlProvider>("SELECT * FROM saml_providers ORDER BY name")
                .fetch_all(self.pool)
                .await?,
        )
    }

    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM saml_providers WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

impl SamlRepository<'_> {
    pub async fn find_by_name(&self, name: &str) -> AppResult<Option<SamlProvider>> {
        Ok(
            sqlx::query_as::<_, SamlProvider>("SELECT * FROM saml_providers WHERE name = $1")
                .bind(name)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    pub async fn find_enabled(&self) -> AppResult<Vec<SamlProvider>> {
        Ok(
            sqlx::query_as::<_, SamlProvider>(
                "SELECT * FROM saml_providers WHERE enabled = true ORDER BY name",
            )
            .fetch_all(self.pool)
            .await?,
        )
    }

    pub async fn create(
        &self,
        name: &str,
        display_name: &str,
        entity_id: &str,
        sso_url: &str,
        certificate: &str,
    ) -> AppResult<SamlProvider> {
        Ok(sqlx::query_as::<_, SamlProvider>(
            "INSERT INTO saml_providers (id, name, display_name, entity_id, sso_url, certificate, enabled, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, true, NOW(), NOW())
             RETURNING *",
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(display_name)
        .bind(entity_id)
        .bind(sso_url)
        .bind(certificate)
        .fetch_one(self.pool)
        .await?)
    }

    pub async fn find_by_entity_id(&self, entity_id: &str) -> AppResult<Option<SamlProvider>> {
        Ok(
            sqlx::query_as::<_, SamlProvider>(
                "SELECT * FROM saml_providers WHERE entity_id = $1",
            )
            .bind(entity_id)
            .fetch_optional(self.pool)
            .await?,
        )
    }

    pub async fn update_enabled(&self, id: Uuid, enabled: bool) -> AppResult<()> {
        sqlx::query("UPDATE saml_providers SET enabled = $1, updated_at = NOW() WHERE id = $2")
            .bind(enabled)
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}
