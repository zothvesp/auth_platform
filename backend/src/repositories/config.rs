use sqlx::PgPool;
use std::collections::HashMap;

use crate::{
    error::AppResult,
    models::{
        config::{keys, FeatureFlags, PasswordPolicy, PublicConfig, ValidationRules},
        SystemConfig,
    },
};

pub struct ConfigRepository<'a> {
    pool: &'a PgPool,
}
impl<'a> ConfigRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }
}

impl ConfigRepository<'_> {
    pub async fn load_all(&self) -> AppResult<HashMap<String, String>> {
        let rows = sqlx::query_as::<_, SystemConfig>("SELECT * FROM system_config")
            .fetch_all(self.pool)
            .await?;
        Ok(rows.into_iter().map(|r| (r.key, r.value)).collect())
    }

    pub async fn load_public(&self) -> AppResult<HashMap<String, String>> {
        let rows =
            sqlx::query_as::<_, SystemConfig>("SELECT * FROM system_config WHERE is_public = true")
                .fetch_all(self.pool)
                .await?;
        Ok(rows.into_iter().map(|r| (r.key, r.value)).collect())
    }

    pub async fn get(&self, key: &str) -> AppResult<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM system_config WHERE key = $1")
                .bind(key)
                .fetch_optional(self.pool)
                .await?;
        Ok(row.map(|r| r.0))
    }

    pub async fn set(pool: &PgPool, key: &str, value: &str) -> AppResult<()> {
        sqlx::query("UPDATE system_config SET value = $1, updated_at = NOW() WHERE key = $2")
            .bind(value)
            .bind(key)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn set_many(pool: &PgPool, entries: &[(&str, &str)]) -> AppResult<()> {
        for (key, value) in entries {
            sqlx::query(
                "INSERT INTO system_config (key, value, description, category, is_public) VALUES ($1,$2,'','custom',false)
                 ON CONFLICT (key) DO UPDATE SET value = $2, updated_at = NOW()")
                .bind(key).bind(value).execute(pool).await?;
        }
        Ok(())
    }

    pub async fn list_by_category(&self, category: &str) -> AppResult<Vec<SystemConfig>> {
        Ok(sqlx::query_as::<_, SystemConfig>(
            "SELECT * FROM system_config WHERE category = $1 ORDER BY key",
        )
        .bind(category)
        .fetch_all(self.pool)
        .await?)
    }

    pub async fn public_config(&self) -> AppResult<PublicConfig> {
        let map = self.load_public().await?;
        let b = |k: &str, d: bool| map.get(k).and_then(|v| v.parse().ok()).unwrap_or(d);
        let u = |k: &str, d: u32| map.get(k).and_then(|v| v.parse().ok()).unwrap_or(d);
        Ok(PublicConfig {
            password_policy: PasswordPolicy {
                min_length: u(keys::PASSWORD_MIN_LENGTH, 8),
                require_uppercase: b(keys::PASSWORD_REQUIRE_UPPERCASE, true),
                require_lowercase: b(keys::PASSWORD_REQUIRE_LOWERCASE, true),
                require_number: b(keys::PASSWORD_REQUIRE_NUMBER, true),
                require_special: b(keys::PASSWORD_REQUIRE_SPECIAL, true),
            },
            validation_rules: ValidationRules {
                display_name_min: u(keys::VALIDATION_DISPLAY_NAME_MIN, 2),
                display_name_max: u(keys::VALIDATION_DISPLAY_NAME_MAX, 50),
                role_name_min: u(keys::VALIDATION_ROLE_NAME_MIN, 2),
                role_name_max: u(keys::VALIDATION_ROLE_NAME_MAX, 50),
            },
            features: FeatureFlags {
                allow_registration: b(keys::ALLOW_REGISTRATION, true),
                require_email_verification: b(keys::REQUIRE_EMAIL_VERIFICATION, true),
                oauth_google: b(keys::OAUTH_GOOGLE_ENABLED, false),
                oauth_github: b(keys::OAUTH_GITHUB_ENABLED, false),
                oauth_microsoft: b(keys::OAUTH_MICROSOFT_ENABLED, false),
                saml_enabled: b(keys::SAML_ENABLED, false),
                mfa_enabled: b(keys::MFA_ENABLED, true),
                mfa_enforce_for_admins: b(keys::MFA_ENFORCE_FOR_ADMINS, false),
            },
        })
    }

    pub async fn get_u64(&self, key: &str, default: u64) -> u64 {
        self.get(key)
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }

    pub async fn get_bool(&self, key: &str, default: bool) -> bool {
        self.get(key)
            .await
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
            .unwrap_or(default)
    }
}
