use anyhow::Context;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,

    // JWT
    pub jwt_private_key: String,  // RS256 PEM private key
    pub jwt_public_key: String,   // RS256 PEM public key
    pub jwt_private_key_previous: Option<String>, // previous private key for rotation
    pub jwt_public_key_previous: Option<String>,  // previous public key for rotation
    pub jwt_access_expiry_secs: u64,
    pub jwt_refresh_expiry_secs: u64,

    // Cookie
    pub cookie_domain: String,
    pub cookie_secure: bool,

    // CORS
    pub allowed_origins: Vec<String>,

    // OAuth providers
    pub google_client_id: String,
    pub google_client_secret: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub microsoft_client_id: String,
    pub microsoft_client_secret: String,
    pub microsoft_tenant_id: String,

    // OAuth redirect base URL
    pub oauth_redirect_base: String,

    // Email
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub smtp_from: String,
    pub app_base_url: String,

    // Security
    pub bcrypt_cost: u32,
    pub max_login_attempts: u32,
    pub lockout_duration_secs: u64,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("PORT must be a number")?,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),

            // JWT keys are read from PEM files on disk.
            // Set JWT_PRIVATE_KEY_PATH / JWT_PUBLIC_KEY_PATH in .env,
            // or drop private.pem / public.pem into the ./keys/ directory.
            jwt_private_key: std::fs::read_to_string(
                env::var("JWT_PRIVATE_KEY_PATH")
                    .unwrap_or_else(|_| "./keys/private.pem".to_string()),
            )
            .context("Failed to read JWT_PRIVATE_KEY_PATH — run `make keys` to generate")?,
            jwt_public_key: std::fs::read_to_string(
                env::var("JWT_PUBLIC_KEY_PATH").unwrap_or_else(|_| "./keys/public.pem".to_string()),
            )
            .context("Failed to read JWT_PUBLIC_KEY_PATH — run `make keys` to generate")?,
            jwt_private_key_previous: env::var("JWT_PRIVATE_KEY_PREVIOUS_PATH")
                .ok()
                .and_then(|path| std::fs::read_to_string(path).ok()),
            jwt_public_key_previous: env::var("JWT_PUBLIC_KEY_PREVIOUS_PATH")
                .ok()
                .and_then(|path| std::fs::read_to_string(path).ok()),
            jwt_access_expiry_secs: env::var("JWT_ACCESS_EXPIRY_SECS")
                .unwrap_or_else(|_| "900".to_string()) // 15 minutes
                .parse()
                .context("JWT_ACCESS_EXPIRY_SECS must be a number")?,
            jwt_refresh_expiry_secs: env::var("JWT_REFRESH_EXPIRY_SECS")
                .unwrap_or_else(|_| "604800".to_string()) // 7 days
                .parse()
                .context("JWT_REFRESH_EXPIRY_SECS must be a number")?,

            cookie_domain: env::var("COOKIE_DOMAIN").unwrap_or_else(|_| "localhost".to_string()),
            cookie_secure: env::var("COOKIE_SECURE")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),

            allowed_origins: env::var("ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),

            // OAuth
            google_client_id: env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            google_client_secret: env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            github_client_id: env::var("GITHUB_CLIENT_ID").unwrap_or_default(),
            github_client_secret: env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
            microsoft_client_id: env::var("MICROSOFT_CLIENT_ID").unwrap_or_default(),
            microsoft_client_secret: env::var("MICROSOFT_CLIENT_SECRET").unwrap_or_default(),
            microsoft_tenant_id: env::var("MICROSOFT_TENANT_ID")
                .unwrap_or_else(|_| "common".to_string()),
            oauth_redirect_base: env::var("OAUTH_REDIRECT_BASE")
                .unwrap_or_else(|_| "http://localhost:3000/auth/callback".to_string()),

            // Email
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "localhost".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "587".to_string())
                .parse()
                .unwrap_or(587),
            smtp_username: env::var("SMTP_USERNAME").unwrap_or_default(),
            smtp_password: env::var("SMTP_PASSWORD").unwrap_or_default(),
            smtp_from: env::var("SMTP_FROM")
                .unwrap_or_else(|_| "noreply@authforge.dev".to_string()),
            app_base_url: env::var("APP_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),

            // Security
            bcrypt_cost: env::var("ARGON2_COST")
                .unwrap_or_else(|_| "3".to_string())
                .parse()
                .unwrap_or(3),
            max_login_attempts: env::var("MAX_LOGIN_ATTEMPTS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            lockout_duration_secs: env::var("LOCKOUT_DURATION_SECS")
                .unwrap_or_else(|_| "900".to_string())
                .parse()
                .unwrap_or(900),
        })
    }
}
