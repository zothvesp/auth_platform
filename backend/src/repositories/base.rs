use crate::error::AppResult;
use crate::models::base::Entity;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Every repository method that touches the DB accepts this executor type.
/// It unifies `&PgPool` (auto-commit) and `&mut Transaction<Postgres>` (manual
/// commit) behind one parameter, enabling transactional composition in services
/// without a Unit of Work registry.
///
/// # Usage in concrete repos
/// ```rust
/// pub async fn create<'e, E>(&self, exec: E, ...) -> AppResult<User>
/// where E: sqlx::PgExecutor<'e>
/// { ... }
/// ```
#[allow(dead_code)]
pub type Db<'e> = &'e PgPool;
#[allow(dead_code)]
pub type Tx<'a> = &'a mut Transaction<'static, Postgres>;

/// Shared read operations every concrete repository provides.
/// Blanket implementations are intentionally NOT provided — each repo
/// writes its own `find_by_id` / `find_all` / `delete` to keep SQL
/// explicit and auditable. This trait is a contract, not magic.
#[async_trait::async_trait]
pub trait BaseRepository {
    type Model: Entity + Send + Unpin;

    async fn find_by_id(&self, id: Uuid) -> AppResult<Option<Self::Model>>;
    async fn find_all(&self) -> AppResult<Vec<Self::Model>>;
    async fn delete(&self, id: Uuid) -> AppResult<()>;

    /// Convenience: find_by_id or 404.
    async fn get(&self, id: Uuid) -> AppResult<Self::Model> {
        use crate::error::AppError;
        self.find_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(std::any::type_name::<Self::Model>().to_string()))
    }

    async fn exists(&self, id: Uuid) -> AppResult<bool> {
        Ok(self.find_by_id(id).await?.is_some())
    }
}
