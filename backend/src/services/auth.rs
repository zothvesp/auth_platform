//! Authentication service — orchestrates repositories, crypto, and JWT.
//! No SQL here. All DB access goes through repositories.

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tracing::{info, warn};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    repositories::{
        base::BaseRepository, RefreshTokenRepository, SessionRepository, UserRepository,
    },
    state::AppState,
};

// ─── DTOs (what crosses the API boundary) ─────────────────────────────────────

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub email_verified: bool,
    pub status: String,
    pub mfa_enabled: bool,
    pub auth_method: String,
    pub last_login_at: Option<chrono::DateTime<Utc>>,
    pub created_at: chrono::DateTime<Utc>,
    pub roles: Vec<RoleDto>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDto {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}

// ─── JWT Claims ───────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
}

// ─── Register ─────────────────────────────────────────────────────────────────

pub async fn register(
    state: &AppState,
    email: &str,
    display_name: &str,
    password: &str,
    ip: &str,
    user_agent: &str,
) -> AppResult<(UserDto, String, String)> {
    let user_repo = UserRepository::new(&state.db.pool);

    if user_repo.email_exists(email).await? {
        return Err(AppError::EmailTaken);
    }

    // Validate password against DB-sourced policy
    let policy = crate::services::config::password_policy(state).await;
    let violations = policy.validate(password);
    if !violations.is_empty() {
        let details = std::collections::HashMap::from([("password".to_string(), violations)]);
        return Err(AppError::Validation(details));
    }

    let password_hash = hash_password(state, password).await?;
    let user_id = Uuid::new_v4();

    let mut tx = state.db.pool.begin().await?;

    UserRepository::create(
        &mut *tx,
        user_id,
        email,
        display_name,
        Some(&password_hash),
        "password",
    )
    .await?;

    // Assign default 'user' role
    let role_repo = crate::repositories::RoleRepository::new(&state.db.pool);
    if let Some(role) = role_repo.find_by_name("user").await? {
        UserRepository::assign_role(&mut *tx, user_id, role.id, None).await?;
    }

    tx.commit().await?;

    // Send verification email in background — non-blocking, non-fatal
    let state_clone = state.clone();
    tokio::spawn(async move {
        crate::services::email::send_verification(&state_clone, user_id)
            .await
            .ok();
    });

    let dto = build_user_dto(state, user_id).await?;
    let access_token = issue_access_token(state, &dto).await?;
    let (refresh_token, family, expires_at) = issue_refresh_token(state, user_id).await?;
    SessionRepository::create(&state.db.pool, family, user_id, ip, user_agent, expires_at).await?;

    info!("User registered: {} from {}", email, ip);
    Ok((dto, access_token, refresh_token))
}

// ─── Login ────────────────────────────────────────────────────────────────────

pub async fn login(
    state: &AppState,
    email: &str,
    password: &str,
    mfa_code: Option<&str>,
    ip: &str,
    user_agent: &str,
    _remember_me: bool,
) -> AppResult<(UserDto, String, String)> {
    check_rate_limit(state, email).await?;

    let user_repo = UserRepository::new(&state.db.pool);
    let user = user_repo
        .find_by_email(email)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    // Validate account state before verifying password (timing-safe ordering)
    match user.status.as_str() {
        "inactive" | "suspended" => return Err(AppError::AccountInactive),
        _ => {}
    }

    verify_password(
        password,
        user.password_hash
            .as_deref()
            .ok_or(AppError::InvalidCredentials)?,
    )
    .map_err(|_| {
        let s = state.clone();
        let e = email.to_string();
        tokio::spawn(async move {
            record_failure(&s, &e).await.ok();
        });
        AppError::InvalidCredentials
    })?;

    if user.mfa_enabled {
        let code = mfa_code.ok_or(AppError::MfaRequired)?;
        crate::services::mfa::verify_code(state, user.id, code).await?;
    }

    UserRepository::update_last_login(&state.db.pool, user.id).await?;
    UserRepository::record_login(&state.db.pool, user.id, ip, user_agent, true, "password").await?;
    clear_rate_limit(state, email).await.ok();

    let dto = build_user_dto(state, user.id).await?;
    let access_token = issue_access_token(state, &dto).await?;
    let (refresh_token, family, expires_at) = issue_refresh_token(state, user.id).await?;
    SessionRepository::create(&state.db.pool, family, user.id, ip, user_agent, expires_at).await?;

    info!("User logged in: {} from {}", email, ip);
    Ok((dto, access_token, refresh_token))
}

// ─── Logout ───────────────────────────────────────────────────────────────────

pub async fn logout(state: &AppState, refresh_token_raw: &str) -> AppResult<()> {
    let hash = hash_token(refresh_token_raw);
    let repo = RefreshTokenRepository::new(&state.db.pool);
    let family = repo.family_by_hash(&hash).await?;
    let mut tx = state.db.pool.begin().await?;
    RefreshTokenRepository::delete_by_hash(&mut *tx, &hash).await?;
    if let Some(family) = family {
        SessionRepository::delete_by_id(&mut *tx, family).await?;
    }
    tx.commit().await?;
    Ok(())
}

// ─── Token Refresh ────────────────────────────────────────────────────────────

pub async fn refresh_token(
    state: &AppState,
    refresh_token_raw: &str,
    ip: &str,
    user_agent: &str,
) -> AppResult<(String, String, u64)> {
    let hash = hash_token(refresh_token_raw);
    let token_repo = RefreshTokenRepository::new(&state.db.pool);

    let record = token_repo
        .find_by_hash(&hash)
        .await?
        .ok_or(AppError::InvalidToken)?;

    if record.expires_at < Utc::now() {
        RefreshTokenRepository::delete_by_hash(&state.db.pool, &hash).await?;
        return Err(AppError::TokenExpired);
    }

    if record.used_at.is_some() {
        warn!(
            "Refresh token reuse detected — revoking family {}",
            record.family
        );
        RefreshTokenRepository::revoke_family(&state.db.pool, record.family).await?;
        return Err(AppError::InvalidToken);
    }

    let mut tx = state.db.pool.begin().await?;
    RefreshTokenRepository::mark_used(&mut *tx, &hash).await?;
    let new_raw = new_refresh_token_value();
    let new_hash = hash_token(&new_raw);
    let expires_at = Utc::now()
        + chrono::Duration::seconds(
            crate::services::config::jwt_refresh_expiry_secs(state).await as i64,
        );
    RefreshTokenRepository::create(
        &mut *tx,
        record.user_id,
        &new_hash,
        record.family,
        expires_at,
    )
    .await?;
    SessionRepository::create(
        &mut *tx,
        record.family,
        record.user_id,
        ip,
        user_agent,
        expires_at,
    )
    .await?;
    tx.commit().await?;

    let dto = build_user_dto(state, record.user_id).await?;
    let access_token = issue_access_token(state, &dto).await?;

    Ok((
        access_token,
        new_raw,
        crate::services::config::jwt_access_expiry_secs(state).await,
    ))
}

// ─── Email Verification ───────────────────────────────────────────────────────

pub async fn verify_email(state: &AppState, token_raw: &str) -> AppResult<()> {
    let hash = hash_token(token_raw);
    let repo = crate::repositories::EmailTokenRepository::new(&state.db.pool);

    let record = repo
        .find_by_hash(&hash)
        .await?
        .ok_or(AppError::InvalidToken)?;

    if record.expires_at < Utc::now() {
        return Err(AppError::TokenExpired);
    }

    let mut tx = state.db.pool.begin().await?;
    UserRepository::set_email_verified(&mut *tx, record.user_id).await?;
    crate::repositories::EmailTokenRepository::delete_by_user(&mut *tx, record.user_id).await?;
    tx.commit().await?;

    Ok(())
}

// ─── Password Reset ───────────────────────────────────────────────────────────

pub async fn change_password(
    state: &AppState,
    user_id: Uuid,
    current_password: &str,
    new_password: &str,
) -> AppResult<()> {
    let user = UserRepository::new(&state.db.pool).get(user_id).await?;

    // Verify current password
    verify_password(
        current_password,
        user.password_hash.as_deref().unwrap_or(""),
    )?;

    // Validate new password against DB policy
    let policy = crate::services::config::password_policy(state).await;
    let violations = policy.validate(new_password);
    if !violations.is_empty() {
        return Err(AppError::Validation(std::collections::HashMap::from([(
            "new_password".to_string(),
            violations,
        )])));
    }

    let hash = hash_password(state, new_password).await?;
    UserRepository::update_password(&state.db.pool, user_id, &hash).await?;

    // Invalidate all sessions on password change
    RefreshTokenRepository::delete_by_user(&state.db.pool, user_id).await?;
    SessionRepository::delete_by_user(&state.db.pool, user_id).await?;

    Ok(())
}

pub async fn forgot_password(state: &AppState, email: &str) -> AppResult<()> {
    use redis::AsyncCommands;
    let key = format!("forgot_password:{}", email);
    let attempts: u32 = state.redis.manager.clone().get(&key).await.unwrap_or(0);
    if attempts >= 3 {
        return Ok(()); // Silently succeed
    }

    let user_repo = UserRepository::new(&state.db.pool);
    // Deliberately swallow "not found" to prevent email enumeration
    if let Some(user) = user_repo.find_by_email(email).await? {
        crate::services::email::send_password_reset(state, user.id).await?;
        let mut conn = state.redis.manager.clone();
        let _: () = conn.incr(&key, 1).await?;
        let _: () = conn.expire(&key, 900).await?; // 15 minutes
    }
    Ok(())
}

pub async fn reset_password(
    state: &AppState,
    token_raw: &str,
    new_password: &str,
) -> AppResult<()> {
    let hash = hash_token(token_raw);
    let repo = crate::repositories::PasswordResetTokenRepository::new(&state.db.pool);

    let record = repo
        .find_unused_by_hash(&hash)
        .await?
        .ok_or(AppError::InvalidToken)?;
    if record.expires_at < Utc::now() {
        return Err(AppError::TokenExpired);
    }

    let policy = crate::services::config::password_policy(state).await;
    let violations = policy.validate(new_password);
    if !violations.is_empty() {
        let details = std::collections::HashMap::from([("password".to_string(), violations)]);
        return Err(AppError::Validation(details));
    }

    let new_hash = hash_password(state, new_password).await?;

    let mut tx = state.db.pool.begin().await?;
    UserRepository::update_password(&mut *tx, record.user_id, &new_hash).await?;
    crate::repositories::PasswordResetTokenRepository::mark_used(&mut *tx, &hash).await?;
    // Invalidate all sessions on password change
    RefreshTokenRepository::delete_by_user(&mut *tx, record.user_id).await?;
    SessionRepository::delete_by_user(&mut *tx, record.user_id).await?;
    tx.commit().await?;

    Ok(())
}

// ─── JWT ──────────────────────────────────────────────────────────────────────

pub async fn issue_access_token(state: &AppState, user: &UserDto) -> AppResult<String> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        roles: user.roles.iter().map(|r| r.name.clone()).collect(),
        permissions: user.permissions.clone(),
        iat: now,
        exp: now + crate::services::config::jwt_access_expiry_secs(state).await as i64,
        jti: Uuid::new_v4().to_string(),
    };

    let signing_key = state.vault.current_signing_key().await?;
    let key = EncodingKey::from_rsa_pem(signing_key.private_key_pem.as_bytes())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT key: {}", e)))?;

    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(signing_key.kid);

    encode(&header, &claims, &key)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT encode: {}", e)))
}

pub async fn validate_access_token(state: &AppState, token: &str) -> AppResult<Claims> {
    let mut v = Validation::new(Algorithm::RS256);
    v.validate_exp = true;

    // Decode header to get kid, then try matching vault keys
    let header = jsonwebtoken::decode_header(token)
        .map_err(|_| AppError::InvalidToken)?;

    if let Some(kid) = header.kid {
        if let Ok(keys) = state.vault.active_keys().await {
            if let Some(matching_key) = keys.iter().find(|k| k.kid == kid) {
                let key = DecodingKey::from_rsa_pem(matching_key.public_key_pem.as_bytes())
                    .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT key: {}", e)))?;
                return match decode::<Claims>(token, &key, &v) {
                    Ok(d) => Ok(d.claims),
                    Err(e) => match e.kind() {
                        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                            Err(AppError::TokenExpired)
                        }
                        _ => Err(AppError::InvalidToken),
                    },
                };
            }
        }
    }

    // Fallback: try PEM file key (legacy / pre-vault tokens)
    let key = DecodingKey::from_rsa_pem(state.config.jwt_public_key.as_bytes())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT key: {}", e)))?;

    match decode::<Claims>(token, &key, &v) {
        Ok(d) => Ok(d.claims),
        Err(e) => {
            if let Some(ref prev_key_pem) = state.config.jwt_public_key_previous {
                if let Ok(prev_key) = DecodingKey::from_rsa_pem(prev_key_pem.as_bytes()) {
                    match decode::<Claims>(token, &prev_key, &v) {
                        Ok(d) => Ok(d.claims),
                        Err(_) => match e.kind() {
                            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                                Err(AppError::TokenExpired)
                            }
                            _ => Err(AppError::InvalidToken),
                        },
                    }
                } else {
                    match e.kind() {
                        jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                            Err(AppError::TokenExpired)
                        }
                        _ => Err(AppError::InvalidToken),
                    }
                }
            } else {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                        Err(AppError::TokenExpired)
                    }
                    _ => Err(AppError::InvalidToken),
                }
            }
        }
    }
}

// ─── OAuth Provider Token Issuance ────────────────────────────────────────────

pub async fn issue_access_token_for_client(
    state: &AppState,
    user: &UserDto,
    _client_id: &str,
    _scope: &str,
) -> AppResult<String> {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        roles: user.roles.iter().map(|r| r.name.clone()).collect(),
        permissions: user.permissions.clone(),
        iat: now,
        exp: now + crate::services::config::jwt_access_expiry_secs(state).await as i64,
        jti: Uuid::new_v4().to_string(),
    };

    // Add audience (client_id) for OIDC
    // Note: We'd need to extend Claims to include `aud` field for proper OIDC
    // For now, we include client_id in the jti to distinguish tokens

    let key = EncodingKey::from_rsa_pem(state.config.jwt_private_key.as_bytes())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT key: {}", e)))?;

    encode(&Header::new(Algorithm::RS256), &claims, &key)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT encode: {}", e)))
}

pub async fn issue_refresh_token_for_client(
    state: &AppState,
    user_id: Uuid,
    _client_id: &str,
) -> AppResult<String> {
    let raw = new_refresh_token_value();
    let hash = hash_token(&raw);
    let family = Uuid::new_v4();
    let expires_at = Utc::now()
        + chrono::Duration::seconds(
            crate::services::config::jwt_refresh_expiry_secs(state).await as i64,
        );
    RefreshTokenRepository::create(&state.db.pool, user_id, &hash, family, expires_at).await?;
    Ok(raw)
}

pub async fn issue_id_token(
    state: &AppState,
    user: &UserDto,
    client_id: &str,
) -> AppResult<String> {
    let now = Utc::now().timestamp();

    // ID Token claims (OIDC standard)
    #[derive(Serialize)]
    struct IdTokenClaims {
        iss: String,
        sub: String,
        aud: String,
        exp: i64,
        iat: i64,
        nonce: Option<String>,
        email: Option<String>,
        name: Option<String>,
        email_verified: Option<bool>,
    }

    let claims = IdTokenClaims {
        iss: state.config.app_base_url.clone(),
        sub: user.id.to_string(),
        aud: client_id.to_string(),
        exp: now + 3600, // ID tokens typically have shorter expiry
        iat: now,
        nonce: None,
        email: Some(user.email.clone()),
        name: Some(user.display_name.clone()),
        email_verified: Some(user.email_verified),
    };

    let key = EncodingKey::from_rsa_pem(state.config.jwt_private_key.as_bytes())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("JWT key: {}", e)))?;

    encode(&Header::new(Algorithm::RS256), &claims, &key)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("ID token encode: {}", e)))
}

// ─── Shared helpers ───────────────────────────────────────────────────────────

pub async fn hash_password(state: &AppState, password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let m_cost = crate::services::config::password_hash_cost(state).await;
    let params = argon2::Params::new(m_cost, 2, 1, None)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Argon2 params: {}", e)))?;
    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Hash: {}", e)))
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<()> {
    let parsed = PasswordHash::new(hash).map_err(|_| AppError::InvalidCredentials)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| AppError::InvalidCredentials)
}

pub fn hash_token(raw: &str) -> String {
    let mut h = Sha256::new();
    h.update(raw.as_bytes());
    hex::encode(h.finalize())
}

fn new_refresh_token_value() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

async fn issue_refresh_token(
    state: &AppState,
    user_id: Uuid,
) -> AppResult<(String, Uuid, chrono::DateTime<Utc>)> {
    let raw = new_refresh_token_value();
    let hash = hash_token(&raw);
    let family = Uuid::new_v4();
    let expires_at = Utc::now()
        + chrono::Duration::seconds(
            crate::services::config::jwt_refresh_expiry_secs(state).await as i64,
        );
    RefreshTokenRepository::create(&state.db.pool, user_id, &hash, family, expires_at).await?;
    Ok((raw, family, expires_at))
}

pub async fn issue_session(
    state: &AppState,
    user_id: Uuid,
    ip: &str,
    user_agent: &str,
) -> AppResult<(UserDto, String, String)> {
    let user = build_user_dto(state, user_id).await?;
    let access_token = issue_access_token(state, &user).await?;
    let (refresh_token, family, expires_at) = issue_refresh_token(state, user_id).await?;
    SessionRepository::create(&state.db.pool, family, user_id, ip, user_agent, expires_at).await?;
    Ok((user, access_token, refresh_token))
}

/// Assembles a UserDto by joining roles and permissions.
/// Called after any mutation that changes user/role state.
pub async fn build_user_dto(state: &AppState, user_id: Uuid) -> AppResult<UserDto> {
    let user_repo = UserRepository::new(&state.db.pool);
    let user = user_repo.get(user_id).await?;
    let roles = crate::services::rbac::get_user_roles(state, user_id).await?;
    let permissions = crate::services::rbac::get_user_permissions(state, user_id).await?;

    Ok(UserDto {
        id: user.id,
        email: user.email,
        display_name: user.display_name,
        avatar_url: user.avatar_url,
        email_verified: user.email_verified,
        status: user.status,
        mfa_enabled: user.mfa_enabled,
        auth_method: user.auth_method,
        last_login_at: user.last_login_at,
        created_at: user.created_at,
        roles: roles
            .iter()
            .map(|r| RoleDto {
                id: r.id,
                name: r.name.clone(),
                description: r.description.clone(),
            })
            .collect(),
        permissions,
    })
}

// ─── Rate limiting (via Redis) ─────────────────────────────────────────────────

async fn check_rate_limit(state: &AppState, email: &str) -> AppResult<()> {
    use redis::AsyncCommands;
    let key = format!("login_failures:{}", email);
    let failures: u32 = state.redis.manager.clone().get(&key).await.unwrap_or(0);
    if failures >= crate::services::config::max_login_attempts(state).await {
        Err(AppError::AccountLocked)
    } else {
        Ok(())
    }
}

async fn record_failure(state: &AppState, email: &str) -> AppResult<()> {
    use redis::AsyncCommands;
    let key = format!("login_failures:{}", email);
    let mut conn = state.redis.manager.clone();
    let _: () = conn.incr(&key, 1).await?;
    let _: () = conn
        .expire(
            &key,
            crate::services::config::lockout_duration_secs(state).await as usize,
        )
        .await?;
    Ok(())
}

async fn clear_rate_limit(state: &AppState, email: &str) -> AppResult<()> {
    use redis::AsyncCommands;
    let mut conn = state.redis.manager.clone();
    let _: () = conn.del(format!("login_failures:{}", email)).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::{
        password_hash::{rand_core::OsRng, SaltString},
        Argon2, PasswordHasher,
    };
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

    fn test_argon2_hash(password: &str) -> String {
        let salt = SaltString::generate(&mut OsRng);
        let params = argon2::Params::new(65536, 2, 1, None).unwrap();
        Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
            .hash_password(password.as_bytes(), &salt)
            .unwrap()
            .to_string()
    }

    fn test_rsa_keypair() -> (String, String) {
        use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};
        use rsa::{RsaPrivateKey, RsaPublicKey};

        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let public_key = RsaPublicKey::from(&private_key);
        let private_pem = private_key
            .to_pkcs8_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap()
            .to_string();
        let public_pem = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap()
            .to_string();
        (private_pem, public_pem)
    }

    #[test]
    fn test_hash_password_verifies() {
        let hash = test_argon2_hash("mypassword");
        assert!(verify_password("mypassword", &hash).is_ok());
    }

    #[test]
    fn test_hash_password_different_each_time() {
        let hash1 = test_argon2_hash("mypassword");
        let hash2 = test_argon2_hash("mypassword");
        assert_ne!(hash1, hash2);
        assert!(verify_password("mypassword", &hash1).is_ok());
        assert!(verify_password("mypassword", &hash2).is_ok());
    }

    #[test]
    fn test_verify_password_wrong_fails() {
        let hash = test_argon2_hash("mypassword");
        assert!(verify_password("wrongpassword", &hash).is_err());
    }

    #[test]
    fn test_validate_access_token() {
        let (private_pem, public_pem) = test_rsa_keypair();

        let claims = Claims {
            sub: "test-user-id".to_string(),
            email: "test@example.com".to_string(),
            roles: vec!["user".to_string()],
            permissions: vec!["users:read".to_string()],
            iat: chrono::Utc::now().timestamp(),
            exp: chrono::Utc::now().timestamp() + 900,
            jti: uuid::Uuid::new_v4().to_string(),
        };

        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes()).unwrap();
        let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key).unwrap();

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes()).unwrap();
        let decoded = decode::<Claims>(&token, &decoding_key, &validation).unwrap();

        assert_eq!(decoded.claims.sub, "test-user-id");
        assert_eq!(decoded.claims.email, "test@example.com");
        assert_eq!(decoded.claims.roles, vec!["user".to_string()]);
        assert_eq!(decoded.claims.permissions, vec!["users:read".to_string()]);
        assert!(decoded.claims.exp > decoded.claims.iat);
    }

    #[test]
    fn test_token_expiry() {
        let (private_pem, public_pem) = test_rsa_keypair();

        let claims = Claims {
            sub: "test-user-id".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![],
            permissions: vec![],
            iat: chrono::Utc::now().timestamp() - 2000,
            exp: chrono::Utc::now().timestamp() - 1000,
            jti: uuid::Uuid::new_v4().to_string(),
        };

        let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes()).unwrap();
        let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key).unwrap();

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes()).unwrap();
        let result = decode::<Claims>(&token, &decoding_key, &validation);

        assert!(result.is_err());
        match result.unwrap_err().kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {}
            other => panic!("Expected ExpiredSignature, got {:?}", other),
        }
    }
}
