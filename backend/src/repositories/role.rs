use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    error::AppResult,
    models::{Permission, Role},
    repositories::base::BaseRepository,
};

pub struct RoleRepository<'a> {
    pool: &'a PgPool,
}
impl<'a> RoleRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BaseRepository for RoleRepository<'_> {
    type Model = Role;
    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Role>> {
        Ok(
            sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE id = $1")
                .bind(id)
                .fetch_optional(self.pool)
                .await?,
        )
    }
    async fn find_all(&self) -> AppResult<Vec<Role>> {
        Ok(
            sqlx::query_as::<_, Role>("SELECT * FROM roles ORDER BY name")
                .fetch_all(self.pool)
                .await?,
        )
    }
    async fn delete(&self, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM roles WHERE id = $1")
            .bind(id)
            .execute(self.pool)
            .await?;
        Ok(())
    }
}

impl RoleRepository<'_> {
    pub async fn find_by_name(&self, name: &str) -> AppResult<Option<Role>> {
        Ok(
            sqlx::query_as::<_, Role>("SELECT * FROM roles WHERE name = $1")
                .bind(name)
                .fetch_optional(self.pool)
                .await?,
        )
    }

    pub async fn find_by_user(&self, user_id: Uuid) -> AppResult<Vec<Role>> {
        Ok(sqlx::query_as::<_, Role>(
            "SELECT r.* FROM roles r JOIN user_roles ur ON ur.role_id = r.id WHERE ur.user_id = $1 ORDER BY r.name")
            .bind(user_id).fetch_all(self.pool).await?)
    }

    pub async fn find_ids_with_hierarchy(&self, user_id: Uuid) -> AppResult<Vec<Uuid>> {
        let rows: Vec<(Uuid,)> = sqlx::query_as(
            "WITH RECURSIVE role_hierarchy AS (
                SELECT r.id, r.parent_role_id FROM roles r JOIN user_roles ur ON ur.role_id = r.id WHERE ur.user_id = $1
                UNION ALL
                SELECT r.id, r.parent_role_id FROM roles r JOIN role_hierarchy rh ON rh.parent_role_id = r.id
            )
            SELECT DISTINCT id FROM role_hierarchy")
            .bind(user_id).fetch_all(self.pool).await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn create<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        name: &str,
        description: &str,
        is_system: bool,
        parent_role_id: Option<Uuid>,
    ) -> AppResult<Role> {
        let now = Utc::now();
        Ok(sqlx::query_as::<_, Role>(
            "INSERT INTO roles (id, name, description, is_system, parent_role_id, created_at, updated_at) VALUES ($1,$2,$3,$4,$5,$6,$6) RETURNING *")
            .bind(Uuid::new_v4()).bind(name).bind(description).bind(is_system).bind(parent_role_id).bind(now)
            .fetch_one(exec).await?)
    }

    pub async fn update_description<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        id: Uuid,
        description: &str,
    ) -> AppResult<()> {
        sqlx::query("UPDATE roles SET description = $1, updated_at = NOW() WHERE id = $2")
            .bind(description)
            .bind(id)
            .execute(exec)
            .await?;
        Ok(())
    }

    pub async fn assign_permission<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("INSERT INTO role_permissions (role_id, permission_id) VALUES ($1,$2) ON CONFLICT DO NOTHING").bind(role_id).bind(permission_id).execute(exec).await?;
        Ok(())
    }

    pub async fn remove_permission<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1 AND permission_id = $2")
            .bind(role_id)
            .bind(permission_id)
            .execute(exec)
            .await?;
        Ok(())
    }

    pub async fn remove_all_permissions<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        role_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
            .bind(role_id)
            .execute(exec)
            .await?;
        Ok(())
    }

    pub async fn find_permissions(&self, role_id: Uuid) -> AppResult<Vec<Permission>> {
        Ok(sqlx::query_as::<_, Permission>(
            "SELECT p.* FROM permissions p JOIN role_permissions rp ON rp.permission_id = p.id WHERE rp.role_id = $1 ORDER BY p.resource, p.action")
            .bind(role_id).fetch_all(self.pool).await?)
    }
}
