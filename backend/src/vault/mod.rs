pub mod hashicorp;
pub mod local;

use crate::error::AppResult;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A single JWT signing key with its metadata.
#[derive(Debug, Clone)]
pub struct JwtKey {
    pub id: Uuid,
    pub kid: String,
    pub public_key_pem: String,
    pub private_key_pem: String,
    pub created_at: DateTime<Utc>,
    pub rotated_at: Option<DateTime<Utc>>,
}

/// Core secret-management trait. The only package that holds/returns raw secret bytes.
///
/// Implementations:
/// - `HashicorpVault`: Uses HashiCorp Vault HTTP API (KV v2 + Transit)
/// - `LocalVault`: Fallback using AES-256-GCM with env master key
#[async_trait::async_trait]
pub trait VaultRepository: Send + Sync {
    // ── JWT key management ──────────────────────────────────────────────────

    /// Return the current (most recent) active signing key.
    /// If no key exists, generates one automatically.
    async fn current_signing_key(&self) -> AppResult<JwtKey>;

    /// Return all non-expired keys for JWKS publication.
    async fn active_keys(&self) -> AppResult<Vec<JwtKey>>;

    /// Generate a new RSA key pair and mark it as the current signing key.
    /// The previous key is kept for grace-period verification.
    async fn rotate_signing_key(&self) -> AppResult<JwtKey>;

    // ── Generic secret encryption ───────────────────────────────────────────

    /// Encrypt a plaintext secret. Returns opaque ciphertext safe to store in DB.
    async fn encrypt(&self, plaintext: &[u8]) -> AppResult<Vec<u8>>;

    /// Decrypt ciphertext back to plaintext.
    async fn decrypt(&self, ciphertext: &[u8]) -> AppResult<Vec<u8>>;

    /// Ensure at least one signing key exists. Called on startup.
    async fn ensure_signing_key(&self) -> AppResult<()>;
}

/// Create the appropriate vault implementation based on environment.
///
/// If VAULT_ADDR is set, uses HashiCorp Vault.
/// Otherwise, falls back to LocalVault with VAULT_MASTER_KEY.
pub fn create_vault(pool: sqlx::PgPool) -> Box<dyn VaultRepository> {
    if std::env::var("VAULT_ADDR").is_ok() {
        Box::new(hashicorp::HashicorpVault::new(pool))
    } else {
        Box::new(local::LocalVault::new(pool))
    }
}
