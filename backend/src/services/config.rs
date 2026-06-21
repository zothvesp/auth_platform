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

// ─── Admin config management ──────────────────────────────────────────────────

pub async fn list_all_config(state: &AppState) -> crate::error::AppResult<std::collections::HashMap<String, String>> {
    ConfigRepository::new(&state.db.pool).load_all().await
}

const BLOCKED_KEYS: &[&str] = &[
    "jwt_private_key",
    "jwt_public_key",
    "jwt_private_key_previous",
    "jwt_public_key_previous",
    "smtp_password",
    "google_client_secret",
    "github_client_secret",
    "microsoft_client_secret",
];

pub async fn update_config(state: &AppState, key: &str, value: &str) -> crate::error::AppResult<String> {
    use crate::error::AppError;

    // Safety: block setting secret values via the API (exact match)
    if BLOCKED_KEYS.contains(&key) {
        return Err(AppError::Forbidden);
    }

    ConfigRepository::set(&state.db.pool, key, value).await?;

    ConfigRepository::new(&state.db.pool)
        .get(key)
        .await?
        .ok_or_else(|| AppError::NotFound("config key".to_string()))
}

// ─── Password hashing ──────────────────────────────────────────────────────────

pub async fn password_hash_cost(state: &AppState) -> u32 {
    ConfigRepository::new(&state.db.pool)
        .get_u64(keys::PASSWORD_HASH_COST, 65536)
        .await as u32
}

// ─── GDPR retention ───────────────────────────────────────────────────────────

pub async fn get_retention_days(state: &AppState) -> i64 {
    ConfigRepository::new(&state.db.pool)
        .get_u64("gdpr.retention_days", 365)
        .await as i64
}

pub async fn get_audit_retention_days(state: &AppState) -> i64 {
    ConfigRepository::new(&state.db.pool)
        .get_u64("gdpr.audit_retention_days", 730)
        .await as i64
}

pub async fn get_login_history_retention_days(state: &AppState) -> i64 {
    ConfigRepository::new(&state.db.pool)
        .get_u64("gdpr.login_history_retention_days", 90)
        .await as i64
}

// ─── Password policy (for backend validation) ─────────────────────────────────

pub async fn password_min_length(state: &AppState) -> usize {
    ConfigRepository::new(&state.db.pool)
        .get_u64(keys::PASSWORD_MIN_LENGTH, 8)
        .await as usize
}

pub async fn password_policy(state: &AppState) -> PasswordPolicySettings {
    let repo = ConfigRepository::new(&state.db.pool);
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

#[cfg(test)]
mod tests {
    use super::PasswordPolicySettings;

    fn default_policy() -> PasswordPolicySettings {
        PasswordPolicySettings {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_number: true,
            require_special: true,
        }
    }

    #[test]
    fn test_password_policy_validate() {
        let policy = default_policy();

        assert!(policy.validate("Strong1!a").is_empty());

        let errors = policy.validate("Str1!a");
        assert!(errors.iter().any(|e| e.contains("at least")));

        let errors = policy.validate("strong1!a");
        assert!(errors.iter().any(|e| e.contains("uppercase")));

        let errors = policy.validate("STRONG1!A");
        assert!(errors.iter().any(|e| e.contains("lowercase")));

        let errors = policy.validate("Strong!!a");
        assert!(errors.iter().any(|e| e.contains("number")));

        let errors = policy.validate("Strong1aB");
        assert!(errors.iter().any(|e| e.contains("special")));

        let errors = policy.validate("short");
        assert!(errors.len() >= 3);
    }

    #[test]
    fn test_password_policy_lenient() {
        let policy = PasswordPolicySettings {
            min_length: 1,
            require_uppercase: false,
            require_lowercase: false,
            require_number: false,
            require_special: false,
        };

        assert!(policy.validate("any").is_empty());
        assert!(policy.validate("a").is_empty());
    }
}
