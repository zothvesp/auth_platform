use crate::{error::AppError, services::auth::validate_access_token, state::AppState};
use axum::body::Body;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tower::{Layer, Service};
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
        let claims = validate_access_token(state, token).await?;

        Ok(AuthUser {
            user_id: claims.sub.parse().map_err(|_| AppError::InvalidToken)?,
            email: claims.email,
            permissions: claims.permissions,
            roles: claims.roles,
        })
    }
}

// ─── Route-level permission guard ─────────────────────────────────────────────

/// Axum middleware that rejects requests lacking a specific permission.
///
/// Apply to a router (or group of routes) to enforce a coarse-grained RBAC
/// gate.  Individual handlers can still call `AuthUser::require_permission`
/// for finer-grained checks — the middleware is additive, not a replacement.
///
/// ```ignore
/// admin::router().layer(axum::middleware::from_fn_with_state(
///     state.clone(),
///     require_permission_middleware("users:manage"),
/// ))
/// ```
pub async fn require_permission_middleware(
    State(state): State<AppState>,
    req: axum::http::Request<Body>,
    next: Next,
) -> Response {
    let (parts, body) = req.into_parts();

    let auth_user = match AuthUser::from_request_parts(&mut parts.clone(), &state).await {
        Ok(user) => user,
        Err(e) => return e.into_response(),
    };

    let permission = extract_required_permission(&parts);
    if let Some(perm) = permission {
        if let Err(e) = auth_user.require_permission(&perm) {
            return e.into_response();
        }
    }

    let req = axum::http::Request::from_parts(parts, body);
    next.run(req).await
}

fn extract_required_permission(parts: &Parts) -> Option<String> {
    parts.extensions.get::<RequiredPermission>().map(|p| p.0.clone())
}

#[derive(Clone)]
pub struct RequiredPermission(pub String);

// ─── Tower Layer for backward compatibility (wraps permission as extension) ───

#[derive(Clone)]
pub struct RequirePermissionLayer {
    permission: String,
}

impl RequirePermissionLayer {
    pub fn new(permission: impl Into<String>) -> Self {
        Self {
            permission: permission.into(),
        }
    }
}

impl<S> Layer<S> for RequirePermissionLayer {
    type Service = RequirePermission<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RequirePermission {
            inner,
            permission: self.permission.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RequirePermission<S> {
    inner: S,
    permission: String,
}

impl<S> Service<axum::http::Request<Body>> for RequirePermission<S>
where
    S: Service<axum::http::Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: IntoResponse + Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: axum::http::Request<Body>) -> Self::Future {
        let permission = self.permission.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let (mut parts, body) = req.into_parts();
            parts.extensions.insert(RequiredPermission(permission));
            inner.call(axum::http::Request::from_parts(parts, body)).await
        })
    }
}
