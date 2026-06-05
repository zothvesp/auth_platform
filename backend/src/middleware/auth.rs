use crate::{error::AppError, services::auth::validate_access_token, state::AppState};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use uuid::Uuid;

/// Extractor that validates the Bearer token and exposes caller identity.
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
    pub permissions: Vec<String>,
    pub roles: Vec<String>,
}

impl AuthUser {
    /// Guard: returns Forbidden if the caller lacks the given permission.
    pub fn require_permission(&self, permission: &str) -> crate::error::AppResult<()> {
        if self.roles.iter().any(|role| role == "super_admin")
            || self.permissions.iter().any(|p| p == permission)
            || implied_manage_permission(permission)
                .is_some_and(|manage| self.permissions.iter().any(|p| p == &manage))
        {
            Ok(())
        } else {
            Err(AppError::InsufficientPermissions(permission.to_string()))
        }
    }

    pub fn require_any_permission(&self, permissions: &[&str]) -> crate::error::AppResult<()> {
        if permissions
            .iter()
            .any(|permission| self.require_permission(permission).is_ok())
        {
            Ok(())
        } else {
            Err(AppError::InsufficientPermissions(permissions.join(", ")))
        }
    }

    pub fn require_role(&self, role: &str) -> crate::error::AppResult<()> {
        if self.roles.iter().any(|r| r == role) {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}

fn implied_manage_permission(permission: &str) -> Option<String> {
    let (resource, action) = permission.split_once(':')?;
    if action == "manage" {
        None
    } else {
        Some(format!("{resource}:manage"))
    }
}

#[async_trait::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(AppError::Unauthorized)?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(AppError::Unauthorized)?;
        let claims = validate_access_token(state, token)?;

        Ok(AuthUser {
            user_id: claims.sub.parse().map_err(|_| AppError::InvalidToken)?,
            email: claims.email,
            permissions: claims.permissions,
            roles: claims.roles,
        })
    }
}
