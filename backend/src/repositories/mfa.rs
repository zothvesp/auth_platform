use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

pub struct MfaRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> MfaRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    pub async fn replace_backup_codes(
        exec: &mut sqlx::PgConnection,
        user_id: Uuid,
        code_hashes: &[String],
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM backup_codes WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *exec)
            .await?;

        for hash in code_hashes {
            sqlx::query(
                "INSERT INTO backup_codes (id, user_id, code_hash, created_at)
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(Uuid::new_v4())
            .bind(user_id)
            .bind(hash)
            .bind(Utc::now())
            .execute(&mut *exec)
            .await?;
        }

        Ok(())
    }

    pub async fn consume_backup_code(&self, user_id: Uuid, code_hash: &str) -> AppResult<bool> {
        let result = sqlx::query(
            "UPDATE backup_codes SET used_at = $1
             WHERE id = (
               SELECT id FROM backup_codes
               WHERE user_id = $2 AND code_hash = $3 AND used_at IS NULL
               LIMIT 1
             )",
        )
        .bind(Utc::now())
        .bind(user_id)
        .bind(code_hash)
        .execute(self.pool)
        .await?;
        Ok(result.rows_affected() == 1)
    }

    pub async fn delete_backup_codes<'e, E: sqlx::PgExecutor<'e>>(
        exec: E,
        user_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query("DELETE FROM backup_codes WHERE user_id = $1")
            .bind(user_id)
            .execute(exec)
            .await?;
        Ok(())
    }
}
