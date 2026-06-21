//! Admin service — user management operations.
//! No SQL here. All DB access goes through repositories.

use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    repositories::{
        base::BaseRepository, AuthorizationCodeRepository, AuditRepository, ConfigRepository,
        EmailTokenRepository, OAuthRepository, PasswordResetTokenRepository, RefreshTokenRepository,
        RoleRepository, SessionRepository, UserRepository,
    },
    services::{self, auth::UserDto},
    state::AppState,
};

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserInput {
    #[validate(email(message = "Invalid email address"))]
    pub email: String,
    pub display_name: String,
    pub password: String,
    pub email_verified: Option<bool>,
    pub status: Option<String>,
    pub role_ids: Option<Vec<Uuid>>,
}

#[derive(serde::Serialize)]
pub struct PaginatedUsers {
    pub data: Vec<UserDto>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

#[derive(serde::Serialize)]
pub struct PaginatedAuditLogs {
    pub data: Vec<crate::models::AuditLog>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}

// ─── User Management ──────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub async fn list_users(
    state: &AppState,
    page: i64,
    page_size: i64,
    status: Option<&str>,
    search: Option<&str>,
    role_id: Option<Uuid>,
    sort_by: &str,
    sort_dir: &str,
) -> AppResult<PaginatedUsers> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 100);

    let (users, total) = UserRepository::new(&state.db.pool)
        .find_paginated(page, page_size, status, search, role_id, sort_by, sort_dir)
        .await?;

    let mut dtos = Vec::with_capacity(users.len());
    for user in users {
        dtos.push(services::auth::build_user_dto(state, user.id).await?);
    }

    let total_pages = (total as f64 / page_size as f64).ceil() as i64;
    Ok(PaginatedUsers {
        data: dtos,
        total,
        page,
        page_size,
        total_pages,
    })
}

pub async fn create_user(
    state: &AppState,
    input: &CreateUserInput,
    requestor_permissions: &[String],
) -> AppResult<UserDto> {
    let status = input.status.as_deref().unwrap_or("active");
    let role_ids = input.role_ids.as_deref().unwrap_or(&[]);

    validate_status(status)?;

    // Check if elevated permissions are needed
    let needs_elevated = !role_ids.is_empty()
        || input.email_verified.unwrap_or(false)
        || status != "active";
    if needs_elevated && !requestor_permissions.contains(&"users:manage".to_string()) {
        return Err(AppError::InsufficientPermissions("users:manage".to_string()));
    }

    // Check email uniqueness
    if UserRepository::new(&state.db.pool)
        .email_exists(&input.email)
        .await?
    {
        return Err(AppError::EmailTaken);
    }

    // Validate password against DB policy
    let violations = services::config::password_policy(state)
        .await
        .validate(&input.password);
    if !violations.is_empty() {
        return Err(AppError::Validation(std::collections::HashMap::from([(
            "password".to_string(),
            violations,
        )])));
    }

    let user_id = Uuid::new_v4();
    let password_hash = services::auth::hash_password(state, &input.password).await?;
    let mut tx = state.db.pool.begin().await?;

    UserRepository::create(
        &mut *tx,
        user_id,
        &input.email,
        &input.display_name,
        Some(&password_hash),
        "password",
    )
    .await?;

    UserRepository::set_status(&mut *tx, user_id, status).await?;

    if input.email_verified.unwrap_or(false) {
        UserRepository::set_email_verified(&mut *tx, user_id).await?;
    }

    let final_role_ids = if role_ids.is_empty() {
        // Default to "user" role if no roles specified
        RoleRepository::new(&state.db.pool)
            .find_by_name("user")
            .await?
            .map(|role| vec![role.id])
            .unwrap_or_default()
    } else {
        // Verify all requested roles exist
        for role_id in role_ids {
            RoleRepository::new(&state.db.pool).get(*role_id).await?;
        }
        role_ids.to_vec()
    };

    for role_id in final_role_ids {
        UserRepository::assign_role(&mut *tx, user_id, role_id, None).await?;
    }

    tx.commit().await?;

    services::auth::build_user_dto(state, user_id).await
}

pub async fn update_user(
    state: &AppState,
    user_id: Uuid,
    display_name: Option<&str>,
    avatar_url: Option<&str>,
) -> AppResult<UserDto> {
    if let Some(name) = display_name {
        UserRepository::update_profile(
            &state.db.pool,
            user_id,
            name,
            avatar_url.filter(|v| !v.trim().is_empty()),
        )
        .await?;
    }
    services::auth::build_user_dto(state, user_id).await
}

pub async fn deactivate_user(state: &AppState, user_id: Uuid) -> AppResult<()> {
    UserRepository::set_status(&state.db.pool, user_id, "inactive").await?;
    Ok(())
}

pub async fn reactivate_user(state: &AppState, user_id: Uuid) -> AppResult<()> {
    UserRepository::set_status(&state.db.pool, user_id, "active").await?;
    Ok(())
}

pub async fn delete_user(state: &AppState, user_id: Uuid) -> AppResult<()> {
    let pool = &state.db.pool;
    let mut tx = pool.begin().await?;

    SessionRepository::delete_by_user(&mut *tx, user_id).await?;
    RefreshTokenRepository::delete_by_user(&mut *tx, user_id).await?;
    OAuthRepository::delete_by_user(&mut *tx, user_id).await?;
    UserRepository::delete_login_history_by_user(&mut *tx, user_id).await?;
    AuthorizationCodeRepository::delete_by_user(&mut *tx, user_id).await?;
    EmailTokenRepository::delete_by_user(&mut *tx, user_id).await?;
    PasswordResetTokenRepository::delete_by_user(&mut *tx, user_id).await?;

    sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    UserRepository::new(pool).delete(user_id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn assign_role(state: &AppState, user_id: Uuid, role_id: Uuid) -> AppResult<()> {
    UserRepository::assign_role(&state.db.pool, user_id, role_id, None).await?;
    Ok(())
}

pub async fn remove_role(state: &AppState, user_id: Uuid, role_id: Uuid) -> AppResult<()> {
    UserRepository::remove_role(&state.db.pool, user_id, role_id).await?;
    Ok(())
}

// ─── Audit Logs ───────────────────────────────────────────────────────────────

pub async fn list_audit_logs(
    state: &AppState,
    page: i64,
    page_size: i64,
    user_id: Option<Uuid>,
) -> AppResult<PaginatedAuditLogs> {
    let page = page.max(1);
    let page_size = page_size.clamp(1, 200);

    let (logs, total) = AuditRepository::new(&state.db.pool)
        .find_paginated(page, page_size, user_id)
        .await?;

    let total_pages = (total as f64 / page_size as f64).ceil() as i64;
    Ok(PaginatedAuditLogs {
        data: logs,
        total,
        page,
        page_size,
        total_pages,
    })
}

// ─── Display Name Validation ──────────────────────────────────────────────────

pub async fn validate_display_name(state: &AppState, name: &str) -> AppResult<()> {
    let rules = ConfigRepository::new(&state.db.pool)
        .public_config()
        .await?
        .validation_rules;
    let trimmed_len = name.trim().chars().count() as u32;
    let mut errors = Vec::new();

    if trimmed_len < rules.display_name_min {
        errors.push(format!(
            "Display name must be at least {} characters",
            rules.display_name_min
        ));
    }
    if trimmed_len > rules.display_name_max {
        errors.push(format!(
            "Display name must be {} characters or less",
            rules.display_name_max
        ));
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(std::collections::HashMap::from([(
            "displayName".to_string(),
            errors,
        )])))
    }
}

// ─── Status Validation ────────────────────────────────────────────────────────

pub fn validate_status(status: &str) -> AppResult<()> {
    if matches!(status, "active" | "inactive" | "suspended") {
        Ok(())
    } else {
        Err(AppError::Validation(std::collections::HashMap::from([(
            "status".to_string(),
            vec!["Status must be active, inactive, or suspended".to_string()],
        )])))
    }
}

// ─── Bulk Operations ─────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct BulkUserResult {
    pub affected: usize,
}

pub async fn bulk_deactivate_users(state: &AppState, user_ids: &[Uuid]) -> AppResult<BulkUserResult> {
    if user_ids.is_empty() {
        return Err(AppError::Validation(std::collections::HashMap::from([(
            "userIds".to_string(),
            vec!["At least one user ID is required".to_string()],
        )])));
    }
    UserRepository::set_status_by_ids(&state.db.pool, user_ids, "inactive").await?;
    Ok(BulkUserResult {
        affected: user_ids.len(),
    })
}

pub async fn bulk_delete_users(state: &AppState, user_ids: &[Uuid]) -> AppResult<BulkUserResult> {
    if user_ids.is_empty() {
        return Err(AppError::Validation(std::collections::HashMap::from([(
            "userIds".to_string(),
            vec!["At least one user ID is required".to_string()],
        )])));
    }
    for &uid in user_ids {
        delete_user(state, uid).await?;
    }
    Ok(BulkUserResult {
        affected: user_ids.len(),
    })
}
