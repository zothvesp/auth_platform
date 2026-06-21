use axum::{
    extract::{Json, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Router,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, AppResult},
    middleware::auth::AuthUser,
    services::{admin, audit},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // OpenAPI spec (public)
        .route("/openapi.json", get(openapi_spec))
        // User management
        .route("/users", get(list_users).post(create_user))
        .route("/users/:id", get(get_user).put(update_user))
        .route("/users/:id/deactivate", patch(deactivate_user))
        .route("/users/:id/reactivate", patch(reactivate_user))
        .route("/users/:id", delete(delete_user))
        .route("/users/:id/roles", post(assign_role).delete(remove_role))
        .route("/users/bulk-deactivate", post(bulk_deactivate_users))
        .route("/users/bulk-delete", post(bulk_delete_users))
        // Audit
        .route("/audit-logs", get(list_audit_logs))
}

#[derive(Deserialize)]
pub struct UserListQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub q: Option<String>,
    pub role_id: Option<Uuid>,
    pub status: Option<String>,
    pub sort_by: Option<String>,
    pub sort_dir: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
}

#[derive(Deserialize)]
pub struct AuditQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub user_id: Option<Uuid>,
}

pub async fn list_users(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<UserListQuery>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:read")?;
    let sort_by = q.sort_by.as_deref().unwrap_or("created_at");
    let sort_by = match sort_by {
        "email" | "last_login" | "created_at" => sort_by,
        _ => "created_at",
    };
    let sort_dir = q.sort_dir.as_deref().unwrap_or("desc");
    let sort_dir = if sort_dir.eq_ignore_ascii_case("asc") {
        "asc"
    } else {
        "desc"
    };
    let result = admin::list_users(
        &state,
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(20),
        q.status.as_deref(),
        q.q.as_deref(),
        q.role_id,
        sort_by,
        sort_dir,
    )
    .await?;
    Ok(Json(result))
}

pub async fn create_user(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<admin::CreateUserInput>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:create")?;
    req.validate().map_err(|errors| {
        let details = errors
            .field_errors()
            .into_iter()
            .map(|(field, errors)| {
                (
                    field.to_string(),
                    errors
                        .iter()
                        .map(|error| error.message.clone().unwrap_or_default().to_string())
                        .collect(),
                )
            })
            .collect();
        AppError::Validation(details)
    })?;

    let result = admin::create_user(&state, &req, &auth.permissions).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "user.create",
        "user",
        Some(&result.id.to_string()),
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        None,
    )
    .await;

    Ok((StatusCode::CREATED, Json(result)))
}

pub async fn get_user(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:read")?;
    Ok(Json(crate::services::auth::build_user_dto(&state, id).await?))
}

pub async fn update_user(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:update")?;
    if let Some(name) = req.display_name.as_deref() {
        admin::validate_display_name(&state, name).await?;
    }
    let result = admin::update_user(
        &state,
        id,
        req.display_name.as_deref(),
        req.avatar_url.as_deref(),
    )
    .await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "user.update",
        "user",
        Some(&id.to_string()),
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        None,
    )
    .await;

    Ok(Json(result))
}

pub async fn deactivate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:manage")?;
    admin::deactivate_user(&state, id).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "user.deactivate",
        "user",
        Some(&id.to_string()),
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        None,
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn reactivate_user(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:manage")?;
    admin::reactivate_user(&state, id).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "user.reactivate",
        "user",
        Some(&id.to_string()),
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        None,
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_user(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:delete")?;
    admin::delete_user(&state, id).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "user.delete",
        "user",
        Some(&id.to_string()),
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        None,
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn assign_role(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AssignRoleRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:manage")?;
    admin::assign_role(&state, user_id, req.role_id).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "user.role.assign",
        "user",
        Some(&user_id.to_string()),
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        Some(serde_json::json!({ "role_id": req.role_id })),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn remove_role(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AssignRoleRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:manage")?;
    admin::remove_role(&state, user_id, req.role_id).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "user.role.remove",
        "user",
        Some(&user_id.to_string()),
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        Some(serde_json::json!({ "role_id": req.role_id })),
    )
    .await;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_audit_logs(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<AuditQuery>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("audit:read")?;
    let result = admin::list_audit_logs(&state, q.page.unwrap_or(1), q.page_size.unwrap_or(50), q.user_id)
        .await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkUserRequest {
    pub user_ids: Vec<Uuid>,
}

pub async fn bulk_deactivate_users(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<BulkUserRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:manage")?;
    let result = admin::bulk_deactivate_users(&state, &req.user_ids).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "user.bulk_deactivate",
        "user",
        None,
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        Some(serde_json::json!({ "user_ids": req.user_ids, "count": result.affected })),
    )
    .await;

    Ok(Json(result))
}

pub async fn bulk_delete_users(
    State(state): State<AppState>,
    auth: AuthUser,
    headers: HeaderMap,
    Json(req): Json<BulkUserRequest>,
) -> AppResult<impl IntoResponse> {
    auth.require_permission("users:delete")?;
    let result = admin::bulk_delete_users(&state, &req.user_ids).await?;

    audit::record(
        &state,
        Some(auth.user_id),
        Some(&auth.email),
        "user.bulk_delete",
        "user",
        None,
        &crate::utils::extract_ip(&headers),
        &crate::utils::extract_ua(&headers),
        true,
        Some(serde_json::json!({ "user_ids": req.user_ids, "count": result.affected })),
    )
    .await;

    Ok(Json(result))
}

pub async fn openapi_spec() -> impl IntoResponse {
    Json(serde_json::json!({
        "openapi": "3.0.3",
        "info": {
            "title": "AuthForge API",
            "version": "0.1.0",
            "description": "Authentication & Identity Platform API"
        },
        "servers": [
            { "url": "/api/v1", "description": "API v1" }
        ],
        "security": [
            { "BearerAuth": [] }
        ],
        "components": {
            "securitySchemes": {
                "BearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        },
        "paths": {
            "/health": {
                "get": {
                    "summary": "Health check",
                    "tags": ["System"],
                    "responses": { "200": { "description": "OK" } }
                }
            },
            "/auth/register": {
                "post": {
                    "summary": "Register a new user",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["email", "displayName", "password"],
                                    "properties": {
                                        "email": { "type": "string", "format": "email" },
                                        "displayName": { "type": "string", "minLength": 2, "maxLength": 50 },
                                        "password": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": { "description": "User created" },
                        "409": { "description": "Email already taken" },
                        "422": { "description": "Validation error" }
                    }
                }
            },
            "/auth/login": {
                "post": {
                    "summary": "Login",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["email", "password"],
                                    "properties": {
                                        "email": { "type": "string" },
                                        "password": { "type": "string" },
                                        "rememberMe": { "type": "boolean" },
                                        "mfaCode": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": { "description": "Login successful" },
                        "401": { "description": "Invalid credentials" }
                    }
                }
            },
            "/auth/logout": {
                "post": {
                    "summary": "Logout",
                    "tags": ["Auth"],
                    "responses": { "204": { "description": "Logged out" } }
                }
            },
            "/auth/refresh": {
                "post": {
                    "summary": "Refresh access token",
                    "tags": ["Auth"],
                    "security": [],
                    "responses": {
                        "200": { "description": "New tokens" },
                        "401": { "description": "Invalid refresh token" }
                    }
                }
            },
            "/auth/verify/{token}": {
                "get": {
                    "summary": "Verify email",
                    "tags": ["Auth"],
                    "security": [],
                    "parameters": [
                        { "name": "token", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": { "200": { "description": "Email verified" } }
                }
            },
            "/auth/resend-verification": {
                "post": {
                    "summary": "Resend verification email",
                    "tags": ["Auth"],
                    "responses": { "200": { "description": "Verification sent" } }
                }
            },
            "/auth/forgot-password": {
                "post": {
                    "summary": "Request password reset",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["email"],
                                    "properties": { "email": { "type": "string" } }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Reset email sent if account exists" } }
                }
            },
            "/auth/reset-password": {
                "post": {
                    "summary": "Reset password with token",
                    "tags": ["Auth"],
                    "security": [],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["token", "password"],
                                    "properties": {
                                        "token": { "type": "string" },
                                        "password": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Password updated" } }
                }
            },
            "/auth/check-permission": {
                "post": {
                    "summary": "Check single permission",
                    "tags": ["Auth"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["permission"],
                                    "properties": { "permission": { "type": "string", "example": "users:read" } }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Permission check result" } }
                }
            },
            "/auth/check-permissions": {
                "post": {
                    "summary": "Batch check permissions",
                    "tags": ["Auth"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["permissions"],
                                    "properties": {
                                        "permissions": { "type": "array", "items": { "type": "string" } }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Permission check results" } }
                }
            },
            "/auth/mfa/setup": {
                "post": {
                    "summary": "Setup MFA (generate TOTP secret)",
                    "tags": ["Auth"],
                    "responses": { "200": { "description": "QR code and secret" } }
                }
            },
            "/auth/mfa/verify": {
                "post": {
                    "summary": "Enable MFA with TOTP code",
                    "tags": ["Auth"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["code"],
                                    "properties": { "code": { "type": "string" } }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "MFA enabled, backup codes returned" } }
                }
            },
            "/auth/mfa/disable": {
                "post": {
                    "summary": "Disable MFA",
                    "tags": ["Auth"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["code"],
                                    "properties": { "code": { "type": "string" } }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "MFA disabled" } }
                }
            },
            "/auth/oauth/{provider}": {
                "get": {
                    "summary": "Get OAuth authorization URL",
                    "tags": ["OAuth"],
                    "security": [],
                    "parameters": [
                        { "name": "provider", "in": "path", "required": true, "schema": { "type": "string", "enum": ["google", "github", "microsoft"] } }
                    ],
                    "responses": { "200": { "description": "Authorization URL" } }
                }
            },
            "/auth/oauth/{provider}/callback": {
                "post": {
                    "summary": "OAuth callback (exchange code for tokens)",
                    "tags": ["OAuth"],
                    "security": [],
                    "parameters": [
                        { "name": "provider", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["code", "state"],
                                    "properties": {
                                        "code": { "type": "string" },
                                        "state": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Login successful" } }
                }
            },
            "/auth/saml/{provider}/login": {
                "get": {
                    "summary": "Initiate SAML login",
                    "tags": ["SAML"],
                    "security": [],
                    "parameters": [
                        { "name": "provider", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": { "200": { "description": "SAML request URL" } }
                }
            },
            "/auth/saml/callback": {
                "post": {
                    "summary": "SAML callback",
                    "tags": ["SAML"],
                    "security": [],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["samlResponse"],
                                    "properties": {
                                        "samlResponse": { "type": "string" },
                                        "relayState": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Login successful" } }
                }
            },
            "/auth/saml/providers": {
                "get": {
                    "summary": "List SAML providers",
                    "tags": ["SAML", "Admin"],
                    "responses": { "200": { "description": "List of SAML providers" } }
                },
                "post": {
                    "summary": "Create SAML provider",
                    "tags": ["SAML", "Admin"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["name", "displayName", "entityId", "ssoUrl", "certificate"],
                                    "properties": {
                                        "name": { "type": "string" },
                                        "displayName": { "type": "string" },
                                        "entityId": { "type": "string" },
                                        "ssoUrl": { "type": "string", "format": "uri" },
                                        "certificate": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "201": { "description": "Provider created" } }
                }
            },
            "/auth/saml/providers/{id}": {
                "delete": {
                    "summary": "Delete SAML provider",
                    "tags": ["SAML", "Admin"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "204": { "description": "Deleted" } }
                }
            },
            "/auth/saml/providers/{id}/toggle": {
                "post": {
                    "summary": "Toggle SAML provider enabled/disabled",
                    "tags": ["SAML", "Admin"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["enabled"],
                                    "properties": { "enabled": { "type": "boolean" } }
                                }
                            }
                        }
                    },
                    "responses": { "204": { "description": "Toggled" } }
                }
            },
            "/config/public": {
                "get": {
                    "summary": "Public config (password policy, feature flags)",
                    "tags": ["Config"],
                    "security": [],
                    "responses": { "200": { "description": "Public configuration" } }
                }
            },
            "/config": {
                "get": {
                    "summary": "List all config (admin)",
                    "tags": ["Config", "Admin"],
                    "responses": { "200": { "description": "All config rows" } }
                }
            },
            "/config/{key}": {
                "put": {
                    "summary": "Update config value (admin)",
                    "tags": ["Config", "Admin"],
                    "parameters": [
                        { "name": "key", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["value"],
                                    "properties": { "value": { "type": "string" } }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Updated config" } }
                }
            },
            "/users/me": {
                "get": {
                    "summary": "Get current user profile",
                    "tags": ["Users"],
                    "responses": { "200": { "description": "User profile" } }
                },
                "put": {
                    "summary": "Update current user profile",
                    "tags": ["Users"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "displayName": { "type": "string" },
                                        "avatarUrl": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Updated user" } }
                }
            },
            "/users/me/change-password": {
                "post": {
                    "summary": "Change password",
                    "tags": ["Users"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["currentPassword", "newPassword"],
                                    "properties": {
                                        "currentPassword": { "type": "string" },
                                        "newPassword": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "204": { "description": "Password changed" } }
                }
            },
            "/users/me/login-history": {
                "get": {
                    "summary": "Get login history",
                    "tags": ["Users"],
                    "responses": { "200": { "description": "Login history entries" } }
                }
            },
            "/users/me/sessions": {
                "get": {
                    "summary": "List active sessions",
                    "tags": ["Users"],
                    "responses": { "200": { "description": "Active sessions" } }
                }
            },
            "/users/me/sessions/{id}": {
                "delete": {
                    "summary": "Revoke session",
                    "tags": ["Users"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "204": { "description": "Session revoked" } }
                }
            },
            "/users/me/oauth-accounts": {
                "get": {
                    "summary": "List linked OAuth accounts",
                    "tags": ["Users"],
                    "responses": { "200": { "description": "Linked accounts" } }
                }
            },
            "/users/me/oauth-accounts/{provider}": {
                "delete": {
                    "summary": "Unlink OAuth account",
                    "tags": ["Users"],
                    "parameters": [
                        { "name": "provider", "in": "path", "required": true, "schema": { "type": "string" } }
                    ],
                    "responses": { "204": { "description": "Account unlinked" } }
                }
            },
            "/roles": {
                "get": {
                    "summary": "List roles",
                    "tags": ["Roles"],
                    "responses": { "200": { "description": "List of roles" } }
                },
                "post": {
                    "summary": "Create role",
                    "tags": ["Roles"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["name", "description"],
                                    "properties": {
                                        "name": { "type": "string" },
                                        "description": { "type": "string" },
                                        "parentRoleId": { "type": "string", "format": "uuid" },
                                        "permissionIds": { "type": "array", "items": { "type": "string", "format": "uuid" } }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "201": { "description": "Role created" } }
                }
            },
            "/roles/{id}": {
                "get": {
                    "summary": "Get role with permissions",
                    "tags": ["Roles"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "200": { "description": "Role details" } }
                },
                "put": {
                    "summary": "Update role",
                    "tags": ["Roles"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "description": { "type": "string" },
                                        "permissionIds": { "type": "array", "items": { "type": "string", "format": "uuid" } }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Updated role" } }
                },
                "delete": {
                    "summary": "Delete role",
                    "tags": ["Roles"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "204": { "description": "Deleted" } }
                }
            },
            "/roles/{id}/permissions": {
                "get": {
                    "summary": "Get role permissions",
                    "tags": ["Roles"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "200": { "description": "Permission list" } }
                }
            },
            "/roles/{id}/permissions/{permId}": {
                "post": {
                    "summary": "Assign permission to role",
                    "tags": ["Roles"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } },
                        { "name": "permId", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "204": { "description": "Permission assigned" } }
                },
                "delete": {
                    "summary": "Remove permission from role",
                    "tags": ["Roles"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } },
                        { "name": "permId", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "204": { "description": "Permission removed" } }
                }
            },
            "/permissions": {
                "get": {
                    "summary": "List permissions",
                    "tags": ["Permissions"],
                    "responses": { "200": { "description": "List of permissions" } }
                },
                "post": {
                    "summary": "Create permission",
                    "tags": ["Permissions"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["resource", "action", "description"],
                                    "properties": {
                                        "resource": { "type": "string", "example": "users" },
                                        "action": { "type": "string", "example": "read" },
                                        "description": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "201": { "description": "Permission created" } }
                }
            },
            "/permissions/{id}": {
                "get": {
                    "summary": "Get permission",
                    "tags": ["Permissions"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "200": { "description": "Permission details" } }
                },
                "put": {
                    "summary": "Update permission",
                    "tags": ["Permissions"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["resource", "action", "description"],
                                    "properties": {
                                        "resource": { "type": "string" },
                                        "action": { "type": "string" },
                                        "description": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Updated permission" } }
                },
                "delete": {
                    "summary": "Delete permission",
                    "tags": ["Permissions"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "204": { "description": "Deleted" } }
                }
            },
            "/admin/users": {
                "get": {
                    "summary": "List users (admin)",
                    "tags": ["Admin"],
                    "parameters": [
                        { "name": "page", "in": "query", "schema": { "type": "integer" } },
                        { "name": "pageSize", "in": "query", "schema": { "type": "integer" } },
                        { "name": "q", "in": "query", "schema": { "type": "string" } },
                        { "name": "roleId", "in": "query", "schema": { "type": "string", "format": "uuid" } },
                        { "name": "status", "in": "query", "schema": { "type": "string" } }
                    ],
                    "responses": { "200": { "description": "Paginated user list" } }
                },
                "post": {
                    "summary": "Create user (admin)",
                    "tags": ["Admin"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "type": "object" }
                            }
                        }
                    },
                    "responses": { "201": { "description": "User created" } }
                }
            },
            "/admin/users/{id}": {
                "get": {
                    "summary": "Get user (admin)",
                    "tags": ["Admin"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "200": { "description": "User details" } }
                },
                "put": {
                    "summary": "Update user (admin)",
                    "tags": ["Admin"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "displayName": { "type": "string" },
                                        "avatarUrl": { "type": "string" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Updated user" } }
                },
                "delete": {
                    "summary": "Delete user (admin)",
                    "tags": ["Admin"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "204": { "description": "Deleted" } }
                }
            },
            "/admin/users/{id}/deactivate": {
                "patch": {
                    "summary": "Deactivate user",
                    "tags": ["Admin"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "204": { "description": "Deactivated" } }
                }
            },
            "/admin/users/{id}/reactivate": {
                "patch": {
                    "summary": "Reactivate user",
                    "tags": ["Admin"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "204": { "description": "Reactivated" } }
                }
            },
            "/admin/users/{id}/roles": {
                "post": {
                    "summary": "Assign role to user",
                    "tags": ["Admin"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["roleId"],
                                    "properties": { "roleId": { "type": "string", "format": "uuid" } }
                                }
                            }
                        }
                    },
                    "responses": { "204": { "description": "Role assigned" } }
                },
                "delete": {
                    "summary": "Remove role from user",
                    "tags": ["Admin"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["roleId"],
                                    "properties": { "roleId": { "type": "string", "format": "uuid" } }
                                }
                            }
                        }
                    },
                    "responses": { "204": { "description": "Role removed" } }
                }
            },
            "/admin/users/bulk-deactivate": {
                "post": {
                    "summary": "Bulk deactivate users",
                    "tags": ["Admin"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["userIds"],
                                    "properties": { "userIds": { "type": "array", "items": { "type": "string", "format": "uuid" } } }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Deactivation result" } }
                }
            },
            "/admin/users/bulk-delete": {
                "post": {
                    "summary": "Bulk delete users",
                    "tags": ["Admin"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["userIds"],
                                    "properties": { "userIds": { "type": "array", "items": { "type": "string", "format": "uuid" } } }
                                }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Deletion result" } }
                }
            },
            "/admin/audit-logs": {
                "get": {
                    "summary": "List audit logs",
                    "tags": ["Admin"],
                    "parameters": [
                        { "name": "page", "in": "query", "schema": { "type": "integer" } },
                        { "name": "pageSize", "in": "query", "schema": { "type": "integer" } },
                        { "name": "userId", "in": "query", "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "200": { "description": "Paginated audit logs" } }
                }
            },
            "/admin/oauth-apps": {
                "get": {
                    "summary": "List OAuth apps",
                    "tags": ["Admin", "OAuth Apps"],
                    "responses": { "200": { "description": "List of OAuth apps" } }
                },
                "post": {
                    "summary": "Create OAuth app",
                    "tags": ["Admin", "OAuth Apps"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "type": "object" }
                            }
                        }
                    },
                    "responses": { "201": { "description": "OAuth app created" } }
                }
            },
            "/admin/oauth-apps/{id}": {
                "get": {
                    "summary": "Get OAuth app",
                    "tags": ["Admin", "OAuth Apps"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "200": { "description": "OAuth app details" } }
                },
                "put": {
                    "summary": "Update OAuth app",
                    "tags": ["Admin", "OAuth Apps"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": { "type": "object" }
                            }
                        }
                    },
                    "responses": { "200": { "description": "Updated OAuth app" } }
                },
                "delete": {
                    "summary": "Delete OAuth app",
                    "tags": ["Admin", "OAuth Apps"],
                    "parameters": [
                        { "name": "id", "in": "path", "required": true, "schema": { "type": "string", "format": "uuid" } }
                    ],
                    "responses": { "204": { "description": "Deleted" } }
                }
            }
        }
    }))
}
