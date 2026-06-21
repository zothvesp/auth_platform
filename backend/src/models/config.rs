use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// A single runtime-configurable setting stored in the database.
/// Secrets (private keys, SMTP password) are excluded — those stay in env.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SystemConfig {
    pub key: String,
    pub value: String,
    pub description: String,
    pub category: String,
    pub is_public: bool,
    pub updated_at: DateTime<Utc>,
}

/// The config subset the frontend receives unauthenticated.
/// Contains password policy, feature flags, and validation rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicConfig {
    pub password_policy: PasswordPolicy,
    pub validation_rules: ValidationRules,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_number: bool,
    pub require_special: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub display_name_min: u32,
    pub display_name_max: u32,
    pub role_name_min: u32,
    pub role_name_max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub allow_registration: bool,
    pub require_email_verification: bool,
    pub oauth_google: bool,
    pub oauth_github: bool,
    pub oauth_microsoft: bool,
    pub saml_enabled: bool,
    pub mfa_enabled: bool,
    pub mfa_enforce_for_admins: bool,
}

/// Type-safe config key constants — the only place key strings are defined.
pub mod keys {
    pub const JWT_ACCESS_EXPIRY_SECS: &str = "auth.jwt_access_expiry_secs";
    pub const JWT_REFRESH_EXPIRY_SECS: &str = "auth.jwt_refresh_expiry_secs";
    pub const MAX_LOGIN_ATTEMPTS: &str = "auth.max_login_attempts";
    pub const LOCKOUT_DURATION_SECS: &str = "auth.lockout_duration_secs";
    pub const REQUIRE_EMAIL_VERIFICATION: &str = "auth.require_email_verification";
    pub const ALLOW_REGISTRATION: &str = "auth.allow_registration";

    pub const PASSWORD_HASH_COST: &str = "password.hash_cost";
    pub const PASSWORD_MIN_LENGTH: &str = "password.min_length";
    pub const PASSWORD_REQUIRE_UPPERCASE: &str = "password.require_uppercase";
    pub const PASSWORD_REQUIRE_LOWERCASE: &str = "password.require_lowercase";
    pub const PASSWORD_REQUIRE_NUMBER: &str = "password.require_number";
    pub const PASSWORD_REQUIRE_SPECIAL: &str = "password.require_special";

    pub const VALIDATION_DISPLAY_NAME_MIN: &str = "validation.display_name_min";
    pub const VALIDATION_DISPLAY_NAME_MAX: &str = "validation.display_name_max";
    pub const VALIDATION_ROLE_NAME_MIN: &str = "validation.role_name_min";
    pub const VALIDATION_ROLE_NAME_MAX: &str = "validation.role_name_max";

    pub const OAUTH_GOOGLE_ENABLED: &str = "oauth.google_enabled";
    pub const OAUTH_GITHUB_ENABLED: &str = "oauth.github_enabled";
    pub const OAUTH_MICROSOFT_ENABLED: &str = "oauth.microsoft_enabled";
    pub const SAML_ENABLED: &str = "oauth.saml_enabled";
    pub const MFA_ENABLED: &str = "mfa.enabled";
    pub const MFA_ENFORCE_FOR_ADMINS: &str = "mfa.enforce_for_admins";

    pub const EMAIL_VERIFICATION_EXPIRY_HRS: &str = "email.verification_expiry_hrs";
    pub const EMAIL_RESET_EXPIRY_MINS: &str = "email.reset_expiry_mins";
    pub const SESSION_COOKIE_SECURE: &str = "session.cookie_secure";
    pub const SESSION_SAME_SITE: &str = "session.same_site";
}
