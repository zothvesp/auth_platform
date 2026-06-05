//! Config service — typed accessors for all runtime configuration.
//!
//! **Rule**: services never read `state.config.<tunable_field>` directly.
//! They call `config::get_*(&state)` here, which reads DB first, falls back
//! to the env-loaded AppConfig. Secrets (JWT keys, SMTP password) are the
//! only values that stay in env-only AppConfig.

use crate::{
    models::config::{keys, PublicConfig},
    repositories::ConfigRepository,
    state::AppState,
};

// ─── Public config (served to unauthenticated clients) ────────────────────────

pub async fn public_config(state: &AppState) -> crate::error::AppResult<PublicConfig> {
    ConfigRepository::new(&state.db.pool).public_config().await
}

// ─── Auth settings ────────────────────────────────────────────────────────────

pub async fn jwt_access_expiry_secs(state: &AppState) -> u64 {
    ConfigRepository::new(&state.db.pool)
        .get_u64(
            keys::JWT_ACCESS_EXPIRY_SECS,
            state.config.jwt_access_expiry_secs,
        )
        .await
}

pub async fn jwt_refresh_expiry_secs(state: &AppState) -> u64 {
    ConfigRepository::new(&state.db.pool)
        .get_u64(
            keys::JWT_REFRESH_EXPIRY_SECS,
            state.config.jwt_refresh_expiry_secs,
        )
        .await
}

pub async fn max_login_attempts(state: &AppState) -> u32 {
    ConfigRepository::new(&state.db.pool)
        .get_u64(
            keys::MAX_LOGIN_ATTEMPTS,
            state.config.max_login_attempts as u64,
        )
        .await as u32
}

pub async fn lockout_duration_secs(state: &AppState) -> u64 {
    ConfigRepository::new(&state.db.pool)
        .get_u64(
            keys::LOCKOUT_DURATION_SECS,
            state.config.lockout_duration_secs,
        )
        .await
}

pub async fn require_email_verification(state: &AppState) -> bool {
    ConfigRepository::new(&state.db.pool)
        .get_bool(keys::REQUIRE_EMAIL_VERIFICATION, true)
        .await
}

pub async fn allow_registration(state: &AppState) -> bool {
    ConfigRepository::new(&state.db.pool)
        .get_bool(keys::ALLOW_REGISTRATION, true)
        .await
}

// ─── Email token expiry ───────────────────────────────────────────────────────

pub async fn email_verification_expiry_hrs(state: &AppState) -> i64 {
    ConfigRepository::new(&state.db.pool)
        .get_u64(keys::EMAIL_VERIFICATION_EXPIRY_HRS, 24)
        .await as i64
}

pub async fn password_reset_expiry_mins(state: &AppState) -> i64 {
    ConfigRepository::new(&state.db.pool)
        .get_u64(keys::EMAIL_RESET_EXPIRY_MINS, 15)
        .await as i64
}

// ─── Cookie settings ──────────────────────────────────────────────────────────

pub async fn cookie_secure(state: &AppState) -> bool {
    ConfigRepository::new(&state.db.pool)
        .get_bool(keys::SESSION_COOKIE_SECURE, state.config.cookie_secure)
        .await
}

pub async fn same_site_policy(state: &AppState) -> String {
    ConfigRepository::new(&state.db.pool)
        .get(keys::SESSION_SAME_SITE)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "Strict".to_string())
}

// ─── Password policy (for backend validation) ─────────────────────────────────

pub async fn password_min_length(state: &AppState) -> usize {
    ConfigRepository::new(&state.db.pool)
        .get_u64(keys::PASSWORD_MIN_LENGTH, 8)
        .await as usize
}

pub async fn password_policy(state: &AppState) -> PasswordPolicySettings {
    let repo = ConfigRepository::new(&state.db.pool);
    let _b = |_k: &str, d: bool| -> bool {
        // Blocking is fine here since this is async context — we call .await
        // in the returned struct builder below
        d // placeholder — actual reads below
    };
    let map = repo.load_public().await.unwrap_or_default();
    let get_bool = |k: &str, d: bool| map.get(k).and_then(|v| v.parse().ok()).unwrap_or(d);
    let get_u32 = |k: &str, d: u32| map.get(k).and_then(|v| v.parse().ok()).unwrap_or(d);

    PasswordPolicySettings {
        min_length: get_u32(keys::PASSWORD_MIN_LENGTH, 8),
        require_uppercase: get_bool(keys::PASSWORD_REQUIRE_UPPERCASE, true),
        require_lowercase: get_bool(keys::PASSWORD_REQUIRE_LOWERCASE, true),
        require_number: get_bool(keys::PASSWORD_REQUIRE_NUMBER, true),
        require_special: get_bool(keys::PASSWORD_REQUIRE_SPECIAL, true),
    }
}

#[derive(Debug, Clone)]
pub struct PasswordPolicySettings {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_special: bool,
}

impl PasswordPolicySettings {
    /// Validate a password against the policy. Returns a list of violation messages.
    pub fn validate(&self, password: &str) -> Vec<String> {
        let mut errors = Vec::new();
        if password.len() < self.min_length as usize {
            errors.push(format!(
                "Password must be at least {} characters",
                self.min_length
            ));
        }
        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }
        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }
        if self.require_number && !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one number".to_string());
        }
        if self.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            errors.push("Password must contain at least one special character".to_string());
        }
        errors
    }
}
