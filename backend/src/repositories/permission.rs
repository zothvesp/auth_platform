use crate::{error::AppResult, models::Permission, repositories::base::BaseRepository};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

pub struct PermissionRepository<'a> {
    pool: &'a PgPool,
}
impl<'a> PermissionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BaseRepository for PermissionRepository<'_> {
    type Model = Permission;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Permission>> {
        Ok(
            sqlx::query_as::<_, Permission>("SELECT * FROM permissions WHERE id = $1")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }
    async fn find_all(&self) -> AppResult<Vec<Permission>> {
        Ok(
            sqlx::query_as::<_, Permission>("SELECT * FROM permissions ORDER BY resource, action")
                .fetch_all(self.pool)
                .await?,
        )
    }
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM permissions WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

impl PermissionRepository<'_> {
    pub async fn find_for_roles(&self, role_ids: &[Uuid]) -> AppResult<Vec<Permission>> {
        if role_ids.is_empty() {
            return Ok(vec![]);
        }
        Ok(sqlx::query_as::<_, Permission>(
            "SELECT DISTINCT p.* FROM permissions p JOIN role_permissions rp ON rp.permission_id = p.id WHERE rp.role_id = ANY($1) ORDER BY p.resource, p.action")
            .bind(role_ids).fetch_all(self.pool).await?)
    }

    pub async fn create<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        resource: &str,
        action: &str,
        description: &str,
    ) -> AppResult<Permission> {
        Ok(sqlx::query_as::<_, Permission>(
            "INSERT INTO permissions (id, resource, action, description, created_at) VALUES ($1,$2,$3,$4,$5) RETURNING *")
            .bind(Uuid::new_v4()).bind(resource).bind(action).bind(description).bind(Utc::now())
            .fetch_one(exec).await?)
    }

    pub async fn upsert<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        resource: &str,
        action: &str,
        description: &str,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO permissions (id, resource, action, description, created_at) VALUES ($1,$2,$3,$4,$5) ON CONFLICT (resource, action) DO NOTHING")
            .bind(Uuid::new_v4()).bind(resource).bind(action).bind(description).bind(Utc::now())
            .execute(exec).await?;
        Ok(())
    }

    pub async fn update<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        id: Uuid,
        resource: &str,
        action: &str,
        description: &str,
    ) -> AppResult<Permission> {
        Ok(sqlx::query_as::<_, Permission>(
            "UPDATE permissions SET resource = $1, action = $2, description = $3 WHERE id = $4 RETURNING *")
            .bind(resource).bind(action).bind(description).bind(id)
            .fetch_one(exec).await?)
    }
}
