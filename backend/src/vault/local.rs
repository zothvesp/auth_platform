use crate::error::{AppError, AppResult};
use crate::vault::{JwtKey, VaultRepository};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
use rsa::pkcs1::LineEnding;
use rsa::RsaPrivateKey;
use sqlx::PgPool;
use tracing::info;

/// Local vault using AES-256-GCM for secret encryption and RSA for JWT signing.
///
/// Master key must be a 32-byte hex string in `VAULT_MASTER_KEY` env var.
/// In production, replace with HashiCorp Vault or cloud KMS.
pub struct LocalVault {
    pool: PgPool,
    cipher: Aes256Gcm,
}

impl LocalVault {
    /// Create a new LocalVault. Panics if `VAULT_MASTER_KEY` is missing or invalid.
    pub fn new(pool: PgPool) -> Self {
        let master_key_hex = std::env::var("VAULT_MASTER_KEY")
            .expect("VAULT_MASTER_KEY env var is required (64-char hex string = 32 bytes)");

        let key_bytes = hex::decode(&master_key_hex)
            .expect("VAULT_MASTER_KEY must be valid hex");
        assert_eq!(key_bytes.len(), 32, "VAULT_MASTER_KEY must be 32 bytes (64 hex chars)");

        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .expect("Failed to create AES-256-GCM cipher");

        Self { pool, cipher }
    }
}

#[async_trait::async_trait]
impl VaultRepository for LocalVault {
    async fn current_signing_key(&self) -> AppResult<JwtKey> {
        sqlx::query_as::<_, JwtKeyRow>(
            "SELECT id, kid, public_key_pem, private_key_pem, created_at, rotated_at
             FROM jwt_keys WHERE rotated_at IS NULL
             ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?
        .map(JwtKeyRow::into_key)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No signing key found — run vault init")))
    }

    async fn active_keys(&self) -> AppResult<Vec<JwtKey>> {
        let rows = sqlx::query_as::<_, JwtKeyRow>(
            "SELECT id, kid, public_key_pem, private_key_pem, created_at, rotated_at
             FROM jwt_keys WHERE rotated_at IS NULL
             ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(JwtKeyRow::into_key).collect())
    }

    async fn rotate_signing_key(&self) -> AppResult<JwtKey> {
        let mut tx = self.pool.begin().await?;

        // Mark current key as rotated
        sqlx::query("UPDATE jwt_keys SET rotated_at = NOW() WHERE rotated_at IS NULL")
            .execute(&mut *tx)
            .await?;

        // Generate new RSA key pair
        let mut rng = OsRng;
        let bits = 2048;
        let private_key = RsaPrivateKey::new(&mut rng, bits)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("RSA key gen: {}", e)))?;
        let public_key = rsa::RsaPublicKey::from(&private_key);

        let private_pem = private_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("PEM encode: {}", e)))?;
        let public_pem = public_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("PEM encode: {}", e)))?;

        let kid = format!("authforge-{}", uuid::Uuid::new_v4());

        let row = sqlx::query_as::<_, JwtKeyRow>(
            "INSERT INTO jwt_keys (id, kid, public_key_pem, private_key_pem, created_at, rotated_at)
             VALUES ($1, $2, $3, $4, NOW(), NULL)
             RETURNING id, kid, public_key_pem, private_key_pem, created_at, rotated_at",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(&kid)
        .bind(&public_pem)
        .bind(&*private_pem)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        info!("JWT signing key rotated: kid={}", kid);
        Ok(row.into_key())
    }

    async fn encrypt(&self, plaintext: &[u8]) -> AppResult<Vec<u8>> {
        use aes_gcm::aead::rand_core::RngCore;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Encrypt: {}", e)))?;

        // Prepend nonce (12 bytes) to ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> AppResult<Vec<u8>> {
        if ciphertext.len() < 12 {
            return Err(AppError::Internal(anyhow::anyhow!("Ciphertext too short")));
        }

        let (nonce_bytes, encrypted) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);

        self.cipher
            .decrypt(nonce, encrypted)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Decrypt: {}", e)))
    }

    async fn ensure_signing_key(&self) -> AppResult<()> {
        let exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM jwt_keys WHERE rotated_at IS NULL)",
        )
        .fetch_one(&self.pool)
        .await?;

        if !exists {
            info!("No signing key found, generating initial key...");
            self.rotate_signing_key().await?;
        }
        Ok(())
    }
}

// ── DB row type ───────────────────────────────────────────────────────────────

#[derive(Debug, sqlx::FromRow)]
struct JwtKeyRow {
    id: uuid::Uuid,
    kid: String,
    public_key_pem: String,
    private_key_pem: String,
    created_at: chrono::DateTime<chrono::Utc>,
    rotated_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl JwtKeyRow {
    fn into_key(self) -> JwtKey {
        JwtKey {
            id: self.id,
            kid: self.kid,
            public_key_pem: self.public_key_pem,
            private_key_pem: self.private_key_pem,
            created_at: self.created_at,
            rotated_at: self.rotated_at,
        }
    }
}
