use crate::error::{AppError, AppResult};
use crate::vault::{JwtKey, VaultRepository};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
use rsa::pkcs1::LineEnding;
use rsa::RsaPrivateKey;
use sqlx::PgPool;
use tracing::info;

/// HashiCorp Vault integration using the HTTP API.
///
/// Requires VAULT_ADDR and VAULT_TOKEN env vars.
/// Uses KV v2 for secret storage and Transit engine for encryption.
pub struct HashicorpVault {
    pool: PgPool,
    client: reqwest::Client,
    addr: String,
    token: String,
}

#[derive(serde::Deserialize)]
struct VaultKVResponse {
    data: VaultKVData,
}

#[derive(serde::Deserialize)]
struct VaultKVData {
    data: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct VaultTransitEncryptResponse {
    data: VaultTransitData,
}

#[derive(serde::Deserialize)]
struct VaultTransitData {
    ciphertext: String,
}

#[derive(serde::Deserialize)]
struct VaultTransitDecryptResponse {
    data: VaultTransitDecryptData,
}

#[derive(serde::Deserialize)]
struct VaultTransitDecryptData {
    plaintext: String,
}

impl HashicorpVault {
    /// Create a new HashicorpVault client. Panics if VAULT_ADDR or VAULT_TOKEN missing.
    pub fn new(pool: PgPool) -> Self {
        let addr = std::env::var("VAULT_ADDR")
            .expect("VAULT_ADDR env var is required (e.g. http://localhost:8200)");
        let token = std::env::var("VAULT_TOKEN")
            .expect("VAULT_TOKEN env var is required");

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self { pool, client, addr, token }
    }

    async fn kv_get(&self, path: &str) -> AppResult<serde_json::Value> {
        let url = format!("{}/v1/secret/data/{}", self.addr, path);
        let resp = self.client
            .get(&url)
            .header("X-Vault-Token", &self.token)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Vault KV read: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!(
                "Vault KV read failed ({}): {}", status, body
            )));
        }

        let kv_resp: VaultKVResponse = resp.json().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Vault KV parse: {}", e)))?;

        Ok(kv_resp.data.data)
    }

    async fn kv_put(&self, path: &str, data: &serde_json::Value) -> AppResult<()> {
        let url = format!("{}/v1/secret/data/{}", self.addr, path);
        let body = serde_json::json!({ "data": data });

        let resp = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Vault KV write: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!(
                "Vault KV write failed ({}): {}", status, body
            )));
        }

        Ok(())
    }

    async fn transit_encrypt(&self, plaintext: &[u8]) -> AppResult<String> {
        let url = format!("{}/v1/transit/encrypt/authforge-master", self.addr);
        let b64 = URL_SAFE_NO_PAD.encode(plaintext);
        let body = serde_json::json!({ "plaintext": b64 });

        let resp = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Vault transit encrypt: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!(
                "Vault transit encrypt failed ({}): {}", status, body
            )));
        }

        let encrypt_resp: VaultTransitEncryptResponse = resp.json().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Vault transit parse: {}", e)))?;

        Ok(encrypt_resp.data.ciphertext)
    }

    async fn transit_decrypt(&self, ciphertext: &str) -> AppResult<Vec<u8>> {
        let url = format!("{}/v1/transit/decrypt/authforge-master", self.addr);
        let body = serde_json::json!({ "ciphertext": ciphertext });

        let resp = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Vault transit decrypt: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AppError::Internal(anyhow::anyhow!(
                "Vault transit decrypt failed ({}): {}", status, body
            )));
        }

        let decrypt_resp: VaultTransitDecryptResponse = resp.json().await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Vault transit parse: {}", e)))?;

        URL_SAFE_NO_PAD.decode(&decrypt_resp.data.plaintext)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Base64 decode: {}", e)))
    }
}

#[async_trait::async_trait]
impl VaultRepository for HashicorpVault {
    async fn current_signing_key(&self) -> AppResult<JwtKey> {
        // Try Vault KV first, fall back to DB
        match self.kv_get("authforge/jwt-keys").await {
            Ok(data) => {
                let kid = data.get("current_kid")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let public_key_pem = data.get("public_key_pem")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let private_key_pem = data.get("private_key_pem")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                if !public_key_pem.is_empty() && !private_key_pem.is_empty() {
                    return Ok(JwtKey {
                        id: uuid::Uuid::new_v4(),
                        kid,
                        public_key_pem,
                        private_key_pem,
                        created_at: chrono::Utc::now(),
                        rotated_at: None,
                    });
                }
            }
            Err(e) => {
                tracing::warn!("Vault KV read failed, falling back to DB: {}", e);
            }
        }

        // Fall back to DB
        sqlx::query_as::<_, JwtKeyRow>(
            "SELECT id, kid, public_key_pem, private_key_pem, created_at, rotated_at
             FROM jwt_keys WHERE rotated_at IS NULL
             ORDER BY created_at DESC LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?
        .map(JwtKeyRow::into_key)
        .ok_or_else(|| AppError::Internal(anyhow::anyhow!("No signing key found")))
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
        let mut rng = rand::rngs::OsRng;
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
        let private_pem_str = private_pem.to_string();

        // Store in DB
        let row = sqlx::query_as::<_, JwtKeyRow>(
            "INSERT INTO jwt_keys (id, kid, public_key_pem, private_key_pem, created_at, rotated_at)
             VALUES ($1, $2, $3, $4, NOW(), NULL)
             RETURNING id, kid, public_key_pem, private_key_pem, created_at, rotated_at",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(&kid)
        .bind(&public_pem)
        .bind(&private_pem_str)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        // Also store in Vault KV (best effort)
        let kv_data = serde_json::json!({
            "current_kid": kid,
            "public_key_pem": public_pem,
            "private_key_pem": private_pem_str,
        });
        if let Err(e) = self.kv_put("authforge/jwt-keys", &kv_data).await {
            tracing::warn!("Failed to store JWT keys in Vault KV: {}", e);
        }

        info!("JWT signing key rotated: kid={}", kid);
        Ok(row.into_key())
    }

    async fn encrypt(&self, plaintext: &[u8]) -> AppResult<Vec<u8>> {
        self.transit_encrypt(plaintext).await.map(|s| s.into_bytes())
    }

    async fn decrypt(&self, ciphertext: &[u8]) -> AppResult<Vec<u8>> {
        let ct_str = std::str::from_utf8(ciphertext)
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Invalid ciphertext: {}", e)))?;
        self.transit_decrypt(ct_str).await
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
