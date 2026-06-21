//! OAuth2/OIDC Provider service — authorization server functionality.
//! No SQL here. All DB access goes through repositories.

use base64::{
    engine::general_purpose::{STANDARD as BASE64, URL_SAFE_NO_PAD},
    Engine,
};
use chrono::{Duration, Utc};
use rsa::pkcs8::DecodePublicKey;
use rsa::traits::PublicKeyParts;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    repositories::{
        base::BaseRepository, AuthorizationCodeRepository, OAuthAppRepository,
        RefreshTokenRepository, UserRepository,
    },
    services,
    state::AppState,
};

// ─── Types ────────────────────────────────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct AuthorizationEndpointResponse {
    pub redirect_to: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct AuthorizationRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scope: Option<String>,
    pub state: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TokenRequest {
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub code_verifier: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: String,
    pub id_token: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserInfoResponse {
    pub sub: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub email_verified: Option<bool>,
}

#[derive(Debug, serde::Serialize)]
pub struct OidcConfiguration {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: String,
    pub scopes_supported: Vec<String>,
    pub response_types_supported: Vec<String>,
    pub grant_types_supported: Vec<String>,
    pub subject_types_supported: Vec<String>,
    pub id_token_signing_alg_values_supported: Vec<String>,
    pub token_endpoint_auth_methods_supported: Vec<String>,
    pub claims_supported: Vec<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct JwksResponse {
    pub keys: Vec<JwksKey>,
}

#[derive(Debug, serde::Serialize)]
pub struct JwksKey {
    pub kty: String,
    pub alg: String,
    #[serde(rename = "use")]
    pub use_: String,
    pub kid: String,
    pub n: String,
    pub e: String,
}

// ─── Authorization Endpoint ───────────────────────────────────────────────────

pub async fn authorize(
    state: &AppState,
    params: &AuthorizationRequest,
) -> AppResult<AuthorizationEndpointResponse> {
    // Validate response_type
    if params.response_type != "code" {
        return Err(AppError::OAuth("unsupported response_type".to_string()));
    }

    // Validate client
    let app = OAuthAppRepository::new(&state.db.pool)
        .find_by_client_id(&params.client_id)
        .await?
        .ok_or_else(|| AppError::OAuth("unknown client_id".to_string()))?;

    if !app.is_active {
        return Err(AppError::OAuth("client is disabled".to_string()));
    }

    // Validate redirect_uri
    if !app.redirect_uris.contains(&params.redirect_uri) {
        return Err(AppError::OAuth("invalid redirect_uri".to_string()));
    }

    // Validate scope
    let scope = params.scope.clone().unwrap_or_else(|| "openid".to_string());
    for s in scope.split_whitespace() {
        if !app.allowed_scopes.iter().any(|allowed| allowed == s) {
            return Err(AppError::OAuth(format!("scope '{}' not allowed", s)));
        }
    }

    // Validate PKCE if required
    if app.pkce_required && params.code_challenge.is_none() {
        return Err(AppError::OAuth("PKCE required".to_string()));
    }

    // For now, return a redirect to the consent page
    // In a real implementation, this would redirect to a consent UI
    let redirect_to = format!(
        "/oauth/consent?client_id={}&redirect_uri={}&scope={}&state={}",
        urlencoding::encode(&params.client_id),
        urlencoding::encode(&params.redirect_uri),
        urlencoding::encode(&scope),
        urlencoding::encode(params.state.as_deref().unwrap_or("")),
    );

    Ok(AuthorizationEndpointResponse { redirect_to })
}

// ─── Issue Authorization Code ─────────────────────────────────────────────────

pub async fn issue_code(
    state: &AppState,
    client_id: &str,
    user_id: Uuid,
    redirect_uri: &str,
    scope: &str,
    code_challenge: Option<&str>,
    code_challenge_method: Option<&str>,
) -> AppResult<String> {
    // Generate random code
    let code = generate_code();
    let code_hash = hash_code(&code);

    // Set expiry (10 minutes)
    let expires_at = Utc::now() + Duration::minutes(10);

    AuthorizationCodeRepository::new(&state.db.pool)
        .create(
            &code_hash,
            client_id,
            user_id,
            redirect_uri,
            scope,
            code_challenge,
            code_challenge_method,
            expires_at,
        )
        .await?;

    Ok(code)
}

// ─── Token Endpoint ───────────────────────────────────────────────────────────

pub async fn exchange_code(
    state: &AppState,
    params: &TokenRequest,
) -> AppResult<TokenResponse> {
    if params.grant_type != "authorization_code" {
        return Err(AppError::OAuth("unsupported grant_type".to_string()));
    }

    let code = params
        .code
        .as_deref()
        .ok_or_else(|| AppError::OAuth("missing code".to_string()))?;
    let redirect_uri = params
        .redirect_uri
        .as_deref()
        .ok_or_else(|| AppError::OAuth("missing redirect_uri".to_string()))?;

    let code_hash = hash_code(code);
    let auth_code = AuthorizationCodeRepository::new(&state.db.pool)
        .find_by_hash(&code_hash)
        .await?
        .ok_or_else(|| AppError::OAuth("invalid code".to_string()))?;

    // Validate code
    if auth_code.used_at.is_some() {
        // Revoke all tokens for this client (token reuse detection)
        return Err(AppError::OAuth("code already used".to_string()));
    }

    if auth_code.expires_at < Utc::now() {
        return Err(AppError::OAuth("code expired".to_string()));
    }

    if auth_code.client_id != params.client_id {
        return Err(AppError::OAuth("client_id mismatch".to_string()));
    }

    if auth_code.redirect_uri != redirect_uri {
        return Err(AppError::OAuth("redirect_uri mismatch".to_string()));
    }

    // Validate PKCE
    if let Some(challenge) = &auth_code.code_challenge {
        let verifier = params
            .code_verifier
            .as_deref()
            .ok_or_else(|| AppError::OAuth("code_verifier required".to_string()))?;

        let method = auth_code
            .code_challenge_method
            .as_deref()
            .unwrap_or("S256");

        let valid = match method {
            "S256" => {
                let digest = Sha256::digest(verifier.as_bytes());
                let computed = BASE64.encode(digest);
                computed == *challenge
            }
            "plain" => verifier == challenge,
            _ => false,
        };

        if !valid {
            return Err(AppError::OAuth("invalid code_verifier".to_string()));
        }
    }

    // Mark code as used
    AuthorizationCodeRepository::new(&state.db.pool)
        .mark_used(&code_hash)
        .await?;

    // Get user
    let user = UserRepository::new(&state.db.pool)
        .get(auth_code.user_id)
        .await?;

    // Build UserDto for token issuance
    let user_dto = services::auth::build_user_dto(state, user.id).await?;

    // Issue tokens
    let scope = auth_code.scope.clone();
    let app = OAuthAppRepository::new(&state.db.pool)
        .find_by_client_id(&params.client_id)
        .await?
        .ok_or_else(|| AppError::OAuth("client not found".to_string()))?;

    let access_token = services::auth::issue_access_token_for_client(
        state,
        &user_dto,
        &app.client_id,
        &scope,
    )
    .await?;

    let refresh_token = services::auth::issue_refresh_token_for_client(
        state,
        user_dto.id,
        &app.client_id,
    )
    .await?;

    // Build ID token if openid scope is requested
    let id_token = if scope.contains("openid") {
        Some(
            services::auth::issue_id_token(
                state,
                &user_dto,
                &app.client_id,
            )
            .await?,
        )
    } else {
        None
    };

    Ok(TokenResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: services::config::jwt_access_expiry_secs(state).await,
        refresh_token: Some(refresh_token),
        scope,
        id_token,
    })
}

// ─── UserInfo Endpoint ────────────────────────────────────────────────────────

pub async fn userinfo(
    state: &AppState,
    user_id: Uuid,
    scope: &str,
) -> AppResult<UserInfoResponse> {
    let user = UserRepository::new(&state.db.pool)
        .get(user_id)
        .await?;

    Ok(UserInfoResponse {
        sub: user.id.to_string(),
        email: if scope.contains("email") {
            Some(user.email)
        } else {
            None
        },
        name: if scope.contains("profile") {
            Some(user.display_name)
        } else {
            None
        },
        email_verified: if scope.contains("email") {
            Some(user.email_verified)
        } else {
            None
        },
    })
}

// ─── OIDC Discovery ──────────────────────────────────────────────────────────

pub fn oidc_configuration(state: &AppState) -> OidcConfiguration {
    let base_url = &state.config.app_base_url;
    OidcConfiguration {
        issuer: base_url.clone(),
        authorization_endpoint: format!("{}/oauth/authorize", base_url),
        token_endpoint: format!("{}/oauth/token", base_url),
        userinfo_endpoint: format!("{}/oauth/userinfo", base_url),
        jwks_uri: format!("{}/oauth/jwks", base_url),
        scopes_supported: vec![
            "openid".to_string(),
            "profile".to_string(),
            "email".to_string(),
        ],
        response_types_supported: vec!["code".to_string()],
        grant_types_supported: vec!["authorization_code".to_string(), "refresh_token".to_string()],
        subject_types_supported: vec!["public".to_string()],
        id_token_signing_alg_values_supported: vec!["RS256".to_string()],
        token_endpoint_auth_methods_supported: vec![
            "client_secret_post".to_string(),
            "client_secret_basic".to_string(),
        ],
        claims_supported: vec![
            "sub".to_string(),
            "email".to_string(),
            "name".to_string(),
            "email_verified".to_string(),
        ],
    }
}

// ─── JWKS ─────────────────────────────────────────────────────────────────────

pub async fn jwks(state: &AppState) -> AppResult<JwksResponse> {
    let active_keys = state.vault.active_keys().await?;
    let mut jwks_keys = Vec::with_capacity(active_keys.len());

    for key in active_keys {
        let public_key = rsa::RsaPublicKey::from_public_key_pem(&key.public_key_pem)
            .map_err(|e| anyhow::anyhow!("Invalid public key {}: {}", key.kid, e))?;

        let n = URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
        let e = URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

        jwks_keys.push(JwksKey {
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            use_: "sig".to_string(),
            kid: key.kid,
            n,
            e,
        });
    }

    Ok(JwksResponse { keys: jwks_keys })
}

// ─── Token Introspection (RFC 7662) ──────────────────────────────────────────

#[derive(Debug, serde::Serialize)]
pub struct IntrospectionResponse {
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iat: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iss: Option<String>,
}

pub async fn introspect_token(state: &AppState, token: &str) -> IntrospectionResponse {
    match services::auth::validate_access_token(state, token).await {
        Ok(claims) => IntrospectionResponse {
            active: true,
            sub: Some(claims.sub),
            client_id: None,
            scope: None,
            exp: Some(claims.exp),
            iat: Some(claims.iat),
            iss: Some(state.config.app_base_url.clone()),
        },
        Err(_) => IntrospectionResponse {
            active: false,
            sub: None,
            client_id: None,
            scope: None,
            exp: None,
            iat: None,
            iss: None,
        },
    }
}

// ─── Token Revocation (RFC 7009) ─────────────────────────────────────────────

pub async fn revoke_token(
    state: &AppState,
    token: &str,
    _token_type_hint: Option<&str>,
) -> AppResult<()> {
    let hash = services::auth::hash_token(token);
    let _ = RefreshTokenRepository::delete_by_hash(&state.db.pool, &hash).await;
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn generate_code() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

fn hash_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    BASE64.encode(hasher.finalize())
}
