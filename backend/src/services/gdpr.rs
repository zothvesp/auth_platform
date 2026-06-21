//! GDPR compliance service — data export, deletion, and retention.
//! Article 17: Right to Erasure. Article 20: Right to Data Portability.

use chrono::{Duration, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    error::AppResult,
    repositories::{
        base::BaseRepository, RefreshTokenRepository, SessionRepository, UserRepository,
    },
    state::AppState,
};

// ─── Data Export (Article 20: Right to Data Portability) ──────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDataExport {
    pub user: serde_json::Value,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub oauth_accounts: Vec<serde_json::Value>,
    pub sessions: Vec<serde_json::Value>,
    pub login_history: Vec<serde_json::Value>,
    pub audit_logs: Vec<serde_json::Value>,
    pub exported_at: chrono::DateTime<Utc>,
}

/// Export all data for a user (GDPR Article 20).
pub async fn export_user_data(state: &AppState, user_id: Uuid) -> AppResult<UserDataExport> {
    let user_repo = UserRepository::new(&state.db.pool);

    // Get user profile
    let user = user_repo.get(user_id).await?;
    let user_json = serde_json::to_value(&user)
        .map_err(|e| crate::error::AppError::Internal(anyhow::anyhow!("Serialize: {}", e)))?;

    // Get roles
    let roles = sqlx::query_scalar::<_, String>(
        "SELECT r.name FROM roles r
         JOIN user_roles ur ON ur.role_id = r.id
         WHERE ur.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await?;

    // Get permissions
    let permissions = sqlx::query_scalar::<_, String>(
        "SELECT DISTINCT p.resource || ':' || p.action
         FROM permissions p
         JOIN role_permissions rp ON rp.permission_id = p.id
         JOIN user_roles ur ON ur.role_id = rp.role_id
         WHERE ur.user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await?;

    // Get OAuth accounts (redact tokens)
    let oauth_accounts = sqlx::query_as::<_, crate::models::OAuthAccount>(
        "SELECT * FROM oauth_accounts WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await?
    .into_iter()
    .filter_map(|a| {
        let mut v = serde_json::to_value(&a).ok()?;
        if let Some(obj) = v.as_object_mut() {
            obj.remove("access_token");
            obj.remove("refresh_token");
        }
        Some(v)
    })
    .collect();

    // Get sessions
    let sessions = sqlx::query_as::<_, crate::models::Session>(
        "SELECT * FROM sessions WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await?
    .into_iter()
    .filter_map(|s| serde_json::to_value(&s).ok())
    .collect();

    // Get login history
    let login_history = sqlx::query_as::<_, crate::models::audit::LoginHistory>(
        "SELECT * FROM login_history WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await?
    .into_iter()
    .filter_map(|h| serde_json::to_value(&h).ok())
    .collect();

    // Get audit logs
    let audit_logs = sqlx::query_as::<_, crate::models::audit::AuditLog>(
        "SELECT * FROM audit_logs WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&state.db.pool)
    .await?
    .into_iter()
    .filter_map(|l| serde_json::to_value(&l).ok())
    .collect();

    Ok(UserDataExport {
        user: user_json,
        roles,
        permissions,
        oauth_accounts,
        sessions,
        login_history,
        audit_logs,
        exported_at: Utc::now(),
    })
}

// ─── Account Deletion (Article 17: Right to Erasure) ─────────────────────────

/// Soft-delete a user account (GDPR Article 17).
/// Anonymizes PII and marks the record as deleted.
pub async fn soft_delete_user(state: &AppState, user_id: Uuid) -> AppResult<()> {
    let mut tx = state.db.pool.begin().await?;

    // Anonymize PII fields
    let anon_email = format!("deleted-{}@anonymized.local", user_id);
    sqlx::query(
        "UPDATE users SET
           email = $1,
           display_name = 'Deleted User',
           password_hash = NULL,
           avatar_url = NULL,
           mfa_secret = NULL,
           mfa_enabled = false,
           status = 'inactive',
           deleted_at = NOW(),
           updated_at = NOW()
         WHERE id = $2 AND deleted_at IS NULL",
    )
    .bind(&anon_email)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Revoke all sessions
    SessionRepository::delete_by_user(&mut *tx, user_id).await?;

    // Revoke all refresh tokens
    RefreshTokenRepository::delete_by_user(&mut *tx, user_id).await?;

    // Anonymize OAuth accounts
    sqlx::query(
        "UPDATE oauth_accounts SET
           access_token = NULL,
           refresh_token = NULL,
           provider_email = NULL,
           updated_at = NOW()
         WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Anonymize login history IPs
    sqlx::query(
        "UPDATE login_history SET
           ip_address = '0.0.0.0',
           user_agent = 'anonymized'
         WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    // Anonymize audit log IPs
    sqlx::query(
        "UPDATE audit_logs SET
           ip_address = '0.0.0.0',
           user_agent = 'anonymized'
         WHERE user_id = $1",
    )
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

// ─── Data Retention Enforcement ───────────────────────────────────────────────

/// Purge soft-deleted users after retention period (GDPR minimization).
pub async fn purge_deleted_users(state: &AppState) -> AppResult<u64> {
    let retention_days = crate::services::config::get_retention_days(state).await;
    let cutoff = Utc::now() - Duration::days(retention_days);

    let result = sqlx::query(
        "DELETE FROM users WHERE deleted_at IS NOT NULL AND deleted_at < $1",
    )
    .bind(cutoff)
    .execute(&state.db.pool)
    .await?;

    Ok(result.rows_affected())
}

/// Purge old audit logs beyond retention period.
pub async fn purge_old_audit_logs(state: &AppState) -> AppResult<u64> {
    let retention_days = crate::services::config::get_audit_retention_days(state).await;
    let cutoff = Utc::now() - Duration::days(retention_days);

    let result = sqlx::query(
        "DELETE FROM audit_logs WHERE created_at < $1",
    )
    .bind(cutoff)
    .execute(&state.db.pool)
    .await?;

    Ok(result.rows_affected())
}

/// Purge old login history beyond retention period.
pub async fn purge_old_login_history(state: &AppState) -> AppResult<u64> {
    let retention_days = crate::services::config::get_login_history_retention_days(state).await;
    let cutoff = Utc::now() - Duration::days(retention_days);

    let result = sqlx::query(
        "DELETE FROM login_history WHERE created_at < $1",
    )
    .bind(cutoff)
    .execute(&state.db.pool)
    .await?;

    Ok(result.rows_affected())
}
