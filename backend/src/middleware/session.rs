use crate::{error::AppError, state::AppState};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

/// Extractor that authenticates via a session cookie backed by Postgres.
///
/// Reads `session_id` from an HttpOnly cookie, looks up the session in
/// the `sessions` table, and attaches the `user_id` to request extensions
/// if valid.
///
/// This supplements JWT-based auth — handlers can accept either `AuthUser`
/// (JWT) or `SessionUser` (session cookie) depending on the use case.
///
/// ```ignore
/// // Usage:
/// pub async fn handler(session: SessionUser) -> impl IntoResponse {
///     let user_id = session.user_id;
///     // ...
/// }
/// ```
pub struct SessionUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

#[async_trait::async_trait]
impl FromRequestParts<AppState> for SessionUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookie_header = parts
            .headers
            .get(axum::http::header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let session_id_str = cookie_header
            .split(';')
            .find(|s| s.trim().starts_with("session_id="))
            .map(|s| s.trim().trim_start_matches("session_id=").to_string())
            .ok_or(AppError::Unauthorized)?;

        if session_id_str.is_empty() {
            return Err(AppError::Unauthorized);
        }

        let session_id: Uuid = session_id_str
            .parse()
            .map_err(|_| AppError::Unauthorized)?;

        // Look up session in Postgres (consistent with SessionRepository)
        let session = sqlx::query_scalar::<_, Uuid>(
            "SELECT user_id FROM sessions WHERE id = $1 AND expires_at > NOW()",
        )
        .bind(session_id)
        .fetch_optional(&state.db.pool)
        .await
        .map_err(|_| AppError::Unauthorized)?;

        let user_id = session.ok_or(AppError::Unauthorized)?;

        Ok(SessionUser {
            user_id,
            session_id,
        })
    }
}
