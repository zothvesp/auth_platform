//! Session service — orchestrates session and refresh token repos.
//! No SQL here. All DB access goes through repositories.

use uuid::Uuid;

use crate::{
    error::AppResult,
    repositories::{RefreshTokenRepository, SessionRepository},
    state::AppState,
};

/// Revoke a session and its associated refresh tokens.
/// This crosses aggregate boundaries (sessions + refresh_tokens),
/// so it belongs in the service layer, not in a repository.
pub async fn revoke(state: &AppState, user_id: Uuid, session_id: Uuid) -> AppResult<()> {
    let mut tx = state.db.pool.begin().await?;

    // Check session exists and belongs to user
    let session_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM sessions WHERE id = $1 AND user_id = $2)",
    )
    .bind(session_id)
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    if !session_exists {
        return Err(crate::error::AppError::NotFound("session".to_string()));
    }

    // Delete session (within sessions aggregate)
    SessionRepository::delete_by_id(&mut *tx, session_id).await?;

    // Delete associated refresh tokens (cross-aggregate, orchestrated here)
    RefreshTokenRepository::revoke_family_for_user(&mut *tx, session_id, user_id).await?;

    tx.commit().await?;
    Ok(())
}
