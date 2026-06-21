//! AuthForge database seeder
//!
//! Idempotent — safe to run multiple times. Uses ON CONFLICT DO NOTHING
//! everywhere so re-running does not duplicate or overwrite data.
//!
//! Run:  cargo run --bin seed
//! Or:   make seed

use anyhow::{Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use chrono::Utc;
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let url = std::env::var("DATABASE_URL").context("DATABASE_URL not set")?;

    let pool = PgPool::connect(&url)
        .await
        .context("Failed to connect to PostgreSQL")?;

    info!("Connected. Starting seed...");

    seed_permissions(&pool).await?;
    seed_roles(&pool).await?;
    seed_role_permissions(&pool).await?;
    seed_users(&pool).await?;
    seed_oauth_apps(&pool).await?;
    seed_demo_audit_data(&pool).await?;

    info!("✓ Seed complete");
    Ok(())
}

// ─── Permissions ─────────────────────────────────────────────────────────────

async fn seed_permissions(pool: &PgPool) -> Result<()> {
    info!("Seeding permissions...");

    let permissions: &[(&str, &str, &str)] = &[
        // Users
        ("users", "read", "View user profiles and list users"),
        ("users", "create", "Create new user accounts"),
        ("users", "update", "Update user profile information"),
        ("users", "delete", "Delete user accounts"),
        (
            "users",
            "manage",
            "Full user management (all user operations)",
        ),
        // Roles
        ("roles", "read", "View roles and their permissions"),
        ("roles", "create", "Create new roles"),
        (
            "roles",
            "update",
            "Update role details and permission assignments",
        ),
        ("roles", "delete", "Delete non-system roles"),
        (
            "roles",
            "manage",
            "Full role management (all role operations)",
        ),
        // Permissions
        ("permissions", "read", "View all permissions"),
        ("permissions", "manage", "Create and delete permissions"),
        // Audit
        ("audit", "read", "View audit logs and login history"),
        // Settings
        ("settings", "read", "View system configuration"),
        ("settings", "manage", "Update system configuration values"),
        // OAuth apps
        ("oauth_apps", "read", "View registered OAuth applications"),
        (
            "oauth_apps",
            "manage",
            "Register and manage OAuth applications",
        ),
    ];

    for (resource, action, description) in permissions {
        sqlx::query(
            "INSERT INTO permissions (id, resource, action, description, created_at)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (resource, action) DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(resource)
        .bind(action)
        .bind(description)
        .bind(Utc::now())
        .execute(pool)
        .await?;
    }

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM permissions")
        .fetch_one(pool)
        .await?;
    info!("  {} permissions in database", count.0);
    Ok(())
}

// ─── Roles ────────────────────────────────────────────────────────────────────

async fn seed_roles(pool: &PgPool) -> Result<()> {
    info!("Seeding roles...");

    // (name, description, is_system)
    let roles: &[(&str, &str, bool)] = &[
        (
            "super_admin",
            "Full system access — all permission checks are bypassed server-side",
            true,
        ),
        (
            "admin",
            "Administrative access: manage users, roles, settings, and audit logs",
            true,
        ),
        (
            "manager",
            "Manage users and view audit logs; cannot change roles or settings",
            true,
        ),
        (
            "user",
            "Standard authenticated user; no administrative permissions",
            true,
        ),
        (
            "viewer",
            "Read-only access to users, roles, and permissions",
            true,
        ),
    ];

    for (name, description, is_system) in roles {
        sqlx::query(
            "INSERT INTO roles (id, name, description, is_system, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $5)
             ON CONFLICT (name) DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(name)
        .bind(description)
        .bind(is_system)
        .bind(Utc::now())
        .execute(pool)
        .await?;
    }

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM roles")
        .fetch_one(pool)
        .await?;
    info!("  {} roles in database", count.0);
    Ok(())
}

// ─── Role → Permission assignments ───────────────────────────────────────────

async fn seed_role_permissions(pool: &PgPool) -> Result<()> {
    info!("Seeding role permissions...");

    // (role_name, [permission_keys])
    let assignments: &[(&str, &[&str])] = &[
        // admin gets everything except super_admin-only ops
        (
            "admin",
            &[
                "users:read",
                "users:create",
                "users:update",
                "users:delete",
                "users:manage",
                "roles:read",
                "roles:create",
                "roles:update",
                "roles:delete",
                "roles:manage",
                "permissions:read",
                "permissions:manage",
                "audit:read",
                "settings:read",
                "settings:manage",
                "oauth_apps:read",
                "oauth_apps:manage",
            ],
        ),
        // manager: users + audit, no role/settings management
        (
            "manager",
            &["users:read", "users:create", "users:update", "audit:read"],
        ),
        // viewer: read-only across the board
        (
            "viewer",
            &["users:read", "roles:read", "permissions:read", "audit:read"],
        ),
        // user: no explicit permissions (can only access their own profile via /users/me)
        ("user", &[]),
        // super_admin: no explicit permissions — server bypass handles it
        ("super_admin", &[]),
    ];

    for (role_name, perms) in assignments {
        // Look up role id
        let role: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM roles WHERE name = $1")
            .bind(role_name)
            .fetch_optional(pool)
            .await?;

        let Some((role_id,)) = role else {
            tracing::warn!("  Role '{}' not found, skipping", role_name);
            continue;
        };

        for perm_key in *perms {
            let parts: Vec<&str> = perm_key.splitn(2, ':').collect();
            let (resource, action) = (parts[0], parts[1]);

            let perm: Option<(Uuid,)> =
                sqlx::query_as("SELECT id FROM permissions WHERE resource = $1 AND action = $2")
                    .bind(resource)
                    .bind(action)
                    .fetch_optional(pool)
                    .await?;

            if let Some((perm_id,)) = perm {
                sqlx::query(
                    "INSERT INTO role_permissions (role_id, permission_id)
                     VALUES ($1, $2)
                     ON CONFLICT DO NOTHING",
                )
                .bind(role_id)
                .bind(perm_id)
                .execute(pool)
                .await?;
            } else {
                tracing::warn!("  Permission '{}' not found, skipping", perm_key);
            }
        }
        info!("  {} → {} permissions assigned", role_name, perms.len());
    }

    Ok(())
}

// ─── Demo users ───────────────────────────────────────────────────────────────

async fn seed_users(pool: &PgPool) -> Result<()> {
    info!("Seeding demo users...");

    let users: &[(&str, &str, &str, &str)] = &[
        // (email, password, display_name, role)
        (
            "superadmin@authforge.dev",
            "Admin@1234!",
            "Super Admin",
            "super_admin",
        ),
        ("admin@authforge.dev", "Admin@1234!", "Admin User", "admin"),
        (
            "manager@authforge.dev",
            "Admin@1234!",
            "Manager User",
            "manager",
        ),
        ("user@authforge.dev", "Admin@1234!", "Regular User", "user"),
        (
            "viewer@authforge.dev",
            "Admin@1234!",
            "Viewer User",
            "viewer",
        ),
    ];

    let argon2 = Argon2::default();

    for (email, password, display_name, role_name) in users {
        // Skip if already exists
        let exists: (bool,) = sqlx::query_as("SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)")
            .bind(email)
            .fetch_one(pool)
            .await?;

        if exists.0 {
            info!("  {} already exists, skipping", email);
            continue;
        }

        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Hash error: {}", e))?
            .to_string();

        let user_id = Uuid::new_v4();
        let now = Utc::now();

        // Insert user
        sqlx::query(
            "INSERT INTO users
               (id, email, display_name, password_hash, email_verified,
                status, mfa_enabled, auth_method, created_at, updated_at)
             VALUES ($1, $2, $3, $4, true, 'active', false, 'password', $5, $5)",
        )
        .bind(user_id)
        .bind(email)
        .bind(display_name)
        .bind(&hash)
        .bind(now)
        .execute(pool)
        .await?;

        // Assign role
        let role: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM roles WHERE name = $1")
            .bind(role_name)
            .fetch_optional(pool)
            .await?;

        if let Some((role_id,)) = role {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id)
                 VALUES ($1, $2)
                 ON CONFLICT DO NOTHING",
            )
            .bind(user_id)
            .bind(role_id)
            .execute(pool)
            .await?;
        }

        info!("  ✓ {} ({}) — password: {}", email, role_name, password);
    }

    info!("");
    info!("  Demo credentials (change in production!):");
    info!("  ┌─────────────────────────────────────────────────────────┐");
    info!("  │ superadmin@authforge.dev  │ Admin@1234!  │ super_admin  │");
    info!("  │ admin@authforge.dev       │ Admin@1234!  │ admin        │");
    info!("  │ manager@authforge.dev     │ Admin@1234!  │ manager      │");
    info!("  │ user@authforge.dev        │ Admin@1234!  │ user         │");
    info!("  │ viewer@authforge.dev      │ Admin@1234!  │ viewer       │");
    info!("  └─────────────────────────────────────────────────────────┘");

    Ok(())
}

// ─── OAuth Apps ──────────────────────────────────────────────────────────────

async fn seed_oauth_apps(pool: &PgPool) -> Result<()> {
    info!("Seeding OAuth apps...");

    type OAuthAppDef = (&'static str, &'static str, &'static str, &'static [&'static str], &'static [&'static str]);
    let apps: &[OAuthAppDef] = &[
        (
            "authforge-admin",
            "AuthForge Admin Console",
            "Internal admin dashboard OAuth client",
            &["http://localhost:3000/auth/callback/admin"],
            &["authorization_code", "refresh_token"],
        ),
        (
            "authforge-docs",
            "AuthForge API Docs",
            "Internal API documentation OAuth client",
            &["http://localhost:3000/auth/callback/docs"],
            &["authorization_code"],
        ),
    ];

    for (client_id, name, description, redirect_uris, allowed_grants) in apps {
        let exists: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM oauth_apps WHERE client_id = $1)",
        )
        .bind(client_id)
        .fetch_one(pool)
        .await?;

        if exists.0 {
            info!("  {} already exists, skipping", client_id);
            continue;
        }

        // Generate a random client secret and hash it
        let secret: String = (0..48)
            .map(|_| {
                let bytes: [u8; 1] = rand::random();
                format!("{:02x}", bytes[0])
            })
            .collect();
        let secret_hash = {
            use sha2::{Sha256, Digest};
            let mut hasher = Sha256::new();
            hasher.update(secret.as_bytes());
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, hasher.finalize())
        };

        sqlx::query(
            "INSERT INTO oauth_apps (id, client_id, client_secret_hash, name, description,
             redirect_uris, allowed_grants, allowed_scopes, pkce_required, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, ARRAY['openid','profile','email']::text[], true, true, NOW(), NOW())",
        )
        .bind(Uuid::new_v4())
        .bind(client_id)
        .bind(&secret_hash)
        .bind(name)
        .bind(description)
        .bind(redirect_uris)
        .bind(allowed_grants)
        .execute(pool)
        .await?;

        info!("  ✓ {} — secret: {}", client_id, secret);
    }

    info!("");
    info!("  OAuth client secrets (change in production!):");
    info!("  ┌──────────────────────────────────────────────────────────────────┐");
    info!("  │ authforge-admin  — see above for secret                         │");
    info!("  │ authforge-docs   — see above for secret                         │");
    info!("  └──────────────────────────────────────────────────────────────────┘");

    Ok(())
}

// ─── Demo audit/login data ───────────────────────────────────────────────────

async fn seed_demo_audit_data(pool: &PgPool) -> Result<()> {
    info!("Seeding demo audit data...");

    // Get the super_admin user
    let user: Option<(Uuid, String)> = sqlx::query_as(
        "SELECT id, email FROM users WHERE email = 'superadmin@authforge.dev' LIMIT 1",
    )
    .fetch_optional(pool)
    .await?;

    let Some((user_id, user_email)) = user else {
        info!("  No super_admin user found, skipping audit data");
        return Ok(());
    };

    // Check if audit data already exists
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM login_history WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        info!("  Audit data already exists, skipping");
        return Ok(());
    }

    // Seed login history (10 entries over the past week)
    let ips = &[
        "192.168.1.100",
        "10.0.0.50",
        "172.16.0.1",
        "192.168.1.100",
        "10.0.0.50",
        "192.168.1.100",
        "172.16.0.1",
        "192.168.1.100",
        "10.0.0.50",
        "192.168.1.100",
    ];
    let agents = &[
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
    ];

    for (i, (ip, agent)) in ips.iter().zip(agents.iter().cycle()).enumerate() {
        let hours_ago = (i * 16) as i64; // Spread over ~7 days
        let ts = Utc::now() - chrono::Duration::hours(hours_ago);

        sqlx::query(
            "INSERT INTO login_history (id, user_id, ip_address, user_agent, success, auth_method, created_at)
             VALUES ($1, $2, $3, $4, true, 'password', $5)",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(ip)
        .bind(agent)
        .bind(ts)
        .execute(pool)
        .await?;
    }

    // Seed audit logs (5 entries)
    let actions = &[
        ("user", "login", true),
        ("user", "update", true),
        ("role", "assign", true),
        ("settings", "update", true),
        ("user", "login", true),
    ];

    for (i, (resource, action, success)) in actions.iter().enumerate() {
        let hours_ago = (i * 12) as i64;
        let ts = Utc::now() - chrono::Duration::hours(hours_ago);

        sqlx::query(
            "INSERT INTO audit_logs (id, user_id, user_email, action, resource, ip_address, user_agent, success, created_at)
             VALUES ($1, $2, $3, $4, $5, '192.168.1.100', 'Mozilla/5.0', $6, $7)",
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(&user_email)
        .bind(action)
        .bind(resource)
        .bind(success)
        .bind(ts)
        .execute(pool)
        .await?;
    }

    info!("  ✓ 10 login history entries, 5 audit log entries seeded");
    Ok(())
}
