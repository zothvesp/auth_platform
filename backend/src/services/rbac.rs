//! RBAC service — permission evaluation and role management.
//! No SQL here. All DB access goes through repositories.

use std::collections::{HashMap, HashSet};
use tracing::info;
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::{Permission, Role},
    repositories::{base::BaseRepository, PermissionRepository, RoleRepository},
    state::AppState,
};
use sqlx::PgPool;

// ─── Permission resolution (the core of RBAC) ────────────────────────────────

/// All direct roles for a user (no hierarchy expansion).
pub async fn get_user_roles(state: &AppState, user_id: Uuid) -> AppResult<Vec<Role>> {
    RoleRepository::new(&state.db.pool)
        .find_by_user(user_id)
        .await
}

/// All permissions for a user after full role hierarchy expansion.
/// This is the authoritative permission set — use it for all checks.
pub async fn get_user_permissions(state: &AppState, user_id: Uuid) -> AppResult<Vec<String>> {
    let role_ids = RoleRepository::new(&state.db.pool)
        .find_ids_with_hierarchy(user_id)
        .await?;

    if role_ids.is_empty() {
        return Ok(vec![]);
    }

    let permissions = PermissionRepository::new(&state.db.pool)
        .find_for_roles(&role_ids)
        .await?;

    Ok(permissions.iter().map(|p| p.key()).collect())
}

/// Single permission check for a user. Super admins always pass.
pub async fn user_has_permission(
    state: &AppState,
    user_id: Uuid,
    permission: &str,
) -> AppResult<bool> {
    let role_repo = RoleRepository::new(&state.db.pool);

    // Super admin bypass
    if let Some(super_admin) = role_repo.find_by_name("super_admin").await? {
        let user_roles = role_repo.find_by_user(user_id).await?;
        if user_roles.iter().any(|r| r.id == super_admin.id) {
            return Ok(true);
        }
    }

    let perms = get_user_permissions(state, user_id).await?;
    Ok(perms.contains(&permission.to_string()))
}

/// Batch permission check — one DB round-trip, returns a map.
pub async fn batch_check_permissions(
    state: &AppState,
    user_id: Uuid,
    permissions: &[String],
) -> AppResult<HashMap<String, bool>> {
    let user_perms: HashSet<String> = get_user_permissions(state, user_id)
        .await?
        .into_iter()
        .collect();

    Ok(permissions
        .iter()
        .map(|p| (p.clone(), user_perms.contains(p)))
        .collect())
}

// ─── Role management ──────────────────────────────────────────────────────────

pub async fn list_roles(state: &AppState) -> AppResult<Vec<Role>> {
    RoleRepository::new(&state.db.pool).find_all().await
}

pub async fn get_role(state: &AppState, role_id: Uuid) -> AppResult<Role> {
    RoleRepository::new(&state.db.pool).get(role_id).await
}

pub async fn create_role(
    state: &AppState,
    name: &str,
    description: &str,
    parent_role_id: Option<Uuid>,
    permission_ids: &[Uuid],
) -> AppResult<Role> {
    let mut tx = state.db.pool.begin().await?;

    let role = RoleRepository::create(&mut *tx, name, description, false, parent_role_id).await?;

    for &perm_id in permission_ids {
        RoleRepository::assign_permission(&mut *tx, role.id, perm_id).await?;
    }

    tx.commit().await?;
    info!("Role created: {}", name);
    Ok(role)
}

pub async fn delete_role(state: &AppState, role_id: Uuid) -> AppResult<()> {
    let role = RoleRepository::new(&state.db.pool).get(role_id).await?;
    if role.is_system {
        return Err(AppError::Conflict(
            "Cannot delete a system role".to_string(),
        ));
    }
    RoleRepository::new(&state.db.pool).delete(role_id).await
}

pub async fn update_role(
    state: &AppState,
    role_id: Uuid,
    description: Option<&str>,
    permission_ids: Option<Vec<Uuid>>,
) -> AppResult<(Role, Vec<Permission>)> {
    // Verify role exists
    let _role = RoleRepository::new(&state.db.pool).get(role_id).await?;

    let pool = &state.db.pool;
    let mut tx = pool.begin().await?;

    if let Some(desc) = description {
        RoleRepository::update_description(&mut *tx, role_id, desc).await?;
    }

    if let Some(perm_ids) = permission_ids {
        RoleRepository::remove_all_permissions(&mut *tx, role_id).await?;
        for perm_id in perm_ids {
            RoleRepository::assign_permission(&mut *tx, role_id, perm_id).await?;
        }
    }

    tx.commit().await?;

    let updated_role = RoleRepository::new(pool).get(role_id).await?;
    let permissions = RoleRepository::new(pool).find_permissions(role_id).await?;

    Ok((updated_role, permissions))
}

pub async fn get_role_permissions(state: &AppState, role_id: Uuid) -> AppResult<Vec<Permission>> {
    RoleRepository::new(&state.db.pool).find_permissions(role_id).await
}

pub async fn assign_permission(
    state: &AppState,
    role_id: Uuid,
    permission_id: Uuid,
) -> AppResult<()> {
    // Verify role exists
    RoleRepository::new(&state.db.pool).get(role_id).await?;
    // Verify permission exists
    PermissionRepository::new(&state.db.pool).get(permission_id).await?;
    // Assign
    RoleRepository::assign_permission(&state.db.pool, role_id, permission_id).await
}

pub async fn remove_permission(
    state: &AppState,
    role_id: Uuid,
    permission_id: Uuid,
) -> AppResult<()> {
    RoleRepository::remove_permission(&state.db.pool, role_id, permission_id).await
}

pub fn validate_role_name(name: &str) -> AppResult<()> {
    if name.len() < 2
        || name.len() > 50
        || !name.chars().all(|c| c.is_ascii_lowercase() || c == '_')
    {
        let mut d = std::collections::HashMap::new();
        d.insert(
            "name".to_string(),
            vec!["Role name must be 2-50 lowercase letters/underscores".to_string()],
        );
        return Err(AppError::Validation(d));
    }
    Ok(())
}

// ─── Permission management ────────────────────────────────────────────────────

pub async fn list_permissions(state: &AppState) -> AppResult<Vec<Permission>> {
    PermissionRepository::new(&state.db.pool).find_all().await
}

pub async fn get_permission(state: &AppState, permission_id: Uuid) -> AppResult<Permission> {
    PermissionRepository::new(&state.db.pool).get(permission_id).await
}

pub async fn create_permission(
    state: &AppState,
    resource: &str,
    action: &str,
    description: &str,
) -> AppResult<Permission> {
    validate_permission_field("resource", resource)?;
    validate_permission_field("action", action)?;
    PermissionRepository::create(&state.db.pool, resource, action, description).await
}

pub async fn update_permission(
    state: &AppState,
    permission_id: Uuid,
    resource: &str,
    action: &str,
    description: &str,
) -> AppResult<Permission> {
    validate_permission_field("resource", resource)?;
    validate_permission_field("action", action)?;
    PermissionRepository::new(&state.db.pool).get(permission_id).await?;
    PermissionRepository::update(&state.db.pool, permission_id, resource, action, description).await
}

pub async fn delete_permission(state: &AppState, permission_id: Uuid) -> AppResult<()> {
    PermissionRepository::new(&state.db.pool).delete(permission_id).await
}

fn validate_permission_field(field: &str, value: &str) -> AppResult<()> {
    if value.len() < 2 || !value.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
        let mut d = std::collections::HashMap::new();
        d.insert(
            field.to_string(),
            vec![format!(
                "{} must be 2+ lowercase letters/underscores",
                field
            )],
        );
        return Err(AppError::Validation(d));
    }
    Ok(())
}

// ─── Seed default roles and permissions ───────────────────────────────────────

pub async fn seed_defaults(state: &AppState) -> anyhow::Result<()> {
    let pool = &state.db.pool;

    // Permissions: (resource, action, description)
    let perms: &[(&str, &str, &str)] = &[
        ("users", "read", "View users"),
        ("users", "create", "Create users"),
        ("users", "update", "Update users"),
        ("users", "delete", "Delete users"),
        ("users", "manage", "Full user management"),
        ("roles", "read", "View roles"),
        ("roles", "create", "Create roles"),
        ("roles", "update", "Update roles"),
        ("roles", "delete", "Delete roles"),
        ("roles", "manage", "Full role management"),
        ("permissions", "read", "View permissions"),
        ("permissions", "manage", "Full permission management"),
        ("audit", "read", "View audit logs"),
        ("settings", "read", "View settings"),
        ("settings", "manage", "Manage settings"),
        ("oauth_apps", "read", "View OAuth apps"),
        ("oauth_apps", "manage", "Manage OAuth apps"),
    ];

    for (resource, action, description) in perms {
        PermissionRepository::upsert(pool, resource, action, description).await?;
    }

    // Roles: (name, description, is_system)
    let roles: &[(&str, &str)] = &[
        (
            "super_admin",
            "Full system access — all permission checks bypassed",
        ),
        (
            "admin",
            "Administrative access to users, roles, and settings",
        ),
        ("manager", "Manage users and view reports"),
        ("user", "Standard authenticated user"),
        ("viewer", "Read-only access to permitted resources"),
    ];

    for (name, description) in roles {
        RoleRepository::create(pool, name, description, true, None)
            .await
            .ok(); // ok = ignore conflict
    }

    // Wire admin permissions
    assign_role_permissions(
        pool,
        "admin",
        &[
            "users:manage",
            "roles:manage",
            "permissions:manage",
            "audit:read",
            "settings:manage",
            "oauth_apps:manage",
        ],
    )
    .await?;

    // Wire manager permissions
    assign_role_permissions(
        pool,
        "manager",
        &["users:read", "users:update", "audit:read"],
    )
    .await?;

    // Wire viewer permissions
    assign_role_permissions(
        pool,
        "viewer",
        &["users:read", "roles:read", "permissions:read"],
    )
    .await?;

    info!("Default roles and permissions seeded");
    Ok(())
}

async fn assign_role_permissions(
    pool: &PgPool,
    role_name: &str,
    perm_keys: &[&str],
) -> anyhow::Result<()> {
    let role_repo = RoleRepository::new(pool);
    let perm_repo = PermissionRepository::new(pool);

    let Some(role) = role_repo.find_by_name(role_name).await? else {
        return Ok(());
    };

    let all_perms = perm_repo.find_all().await?;
    let perm_map: HashMap<String, Uuid> = all_perms.iter().map(|p| (p.key(), p.id)).collect();

    for key in perm_keys {
        if let Some(&perm_id) = perm_map.get(*key) {
            RoleRepository::assign_permission(pool, role.id, perm_id).await?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_role_name;

    #[test]
    fn test_validate_role_name_valid() {
        assert!(validate_role_name("admin").is_ok());
        assert!(validate_role_name("super_admin").is_ok());
        assert!(validate_role_name("ab").is_ok());
        assert!(validate_role_name("a_very_long_but_valid_role_name").is_ok());
    }

    #[test]
    fn test_validate_role_name_too_short() {
        assert!(validate_role_name("a").is_err());
        assert!(validate_role_name("").is_err());
    }

    #[test]
    fn test_validate_role_name_invalid_chars() {
        assert!(validate_role_name("Admin").is_err());
        assert!(validate_role_name("admin-role").is_err());
        assert!(validate_role_name("admin role").is_err());
        assert!(validate_role_name("admin123").is_err());
    }
}
