use oauth2::{
    basic::BasicClient, reqwest::async_http_client, AuthType, AuthUrl, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
    TokenResponse, TokenUrl,
};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::config::keys,
    repositories::{
        base::BaseRepository, ConfigRepository, OAuthRepository, RoleRepository, UserRepository,
    },
    services,
    state::AppState,
};

const STATE_TTL_SECS: usize = 600;

#[derive(Serialize)]
pub struct AuthorizationResponse {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
struct PendingAuthorization {
    provider: String,
    pkce_verifier: String,
}

#[derive(Debug)]
struct ProviderProfile {
    id: String,
    email: String,
    display_name: String,
}

#[derive(Clone, Copy)]
enum Provider {
    Google,
    Github,
    Microsoft,
}

impl Provider {
    fn parse(value: &str) -> AppResult<Self> {
        match value {
            "google" => Ok(Self::Google),
            "github" => Ok(Self::Github),
            "microsoft" => Ok(Self::Microsoft),
            _ => Err(AppError::NotFound("oauth provider".to_string())),
        }
    }

    fn name(self) -> &'static str {
        match self {
            Self::Google => "google",
            Self::Github => "github",
            Self::Microsoft => "microsoft",
        }
    }

    fn flag_key(self) -> &'static str {
        match self {
            Self::Google => keys::OAUTH_GOOGLE_ENABLED,
            Self::Github => keys::OAUTH_GITHUB_ENABLED,
            Self::Microsoft => keys::OAUTH_MICROSOFT_ENABLED,
        }
    }
}

pub async fn authorization_url(
    state: &AppState,
    provider_name: &str,
) -> AppResult<AuthorizationResponse> {
    let provider = Provider::parse(provider_name)?;
    ensure_enabled(state, provider).await?;
    let client = client(state, provider)?;
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();

    let mut request = client
        .authorize_url(CsrfToken::new_random)
        .set_pkce_challenge(challenge);
    for scope in scopes(provider) {
        request = request.add_scope(Scope::new(scope.to_string()));
    }
    let (url, csrf) = request.url();
    let pending = PendingAuthorization {
        provider: provider.name().to_string(),
        pkce_verifier: verifier.secret().to_string(),
    };
    let encoded = serde_json::to_string(&pending)
        .map_err(|error| AppError::Internal(anyhow::anyhow!("OAuth state: {}", error)))?;
    let mut redis = state.redis.manager.clone();
    let key = state_key(csrf.secret());
    let _: () = redis.set_ex(key, encoded, STATE_TTL_SECS).await?;

    Ok(AuthorizationResponse {
        url: url.to_string(),
    })
}

pub async fn callback(
    state: &AppState,
    provider_name: &str,
    code: &str,
    csrf_state: &str,
    ip: &str,
    user_agent: &str,
) -> AppResult<(services::auth::UserDto, String, String)> {
    let provider = Provider::parse(provider_name)?;
    ensure_enabled(state, provider).await?;
    let pending = consume_state(state, csrf_state).await?;
    if pending.provider != provider.name() {
        return Err(AppError::InvalidToken);
    }

    let token = client(state, provider)?
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(PkceCodeVerifier::new(pending.pkce_verifier))
        .request_async(async_http_client)
        .await
        .map_err(|error| AppError::OAuth(format!("Token exchange failed: {}", error)))?;

    let profile = fetch_profile(provider, token.access_token().secret()).await?;
    let user_id = upsert_identity(state, provider, &profile).await?;
    let user = UserRepository::new(&state.db.pool).get(user_id).await?;
    if user.status != "active" {
        return Err(AppError::AccountInactive);
    }
    if user.mfa_enabled {
        return Err(AppError::MfaRequired);
    }
    UserRepository::update_last_login(&state.db.pool, user_id).await?;
    UserRepository::record_login(
        &state.db.pool,
        user_id,
        ip,
        user_agent,
        true,
        provider.name(),
    )
    .await?;
    services::auth::issue_session(state, user_id, ip, user_agent).await
}

async fn ensure_enabled(state: &AppState, provider: Provider) -> AppResult<()> {
    let enabled = ConfigRepository::new(&state.db.pool)
        .get_bool(provider.flag_key(), false)
        .await;
    let configured = match provider {
        Provider::Google => {
            !state.config.google_client_id.is_empty()
                && !state.config.google_client_secret.is_empty()
        }
        Provider::Github => {
            !state.config.github_client_id.is_empty()
                && !state.config.github_client_secret.is_empty()
        }
        Provider::Microsoft => {
            !state.config.microsoft_client_id.is_empty()
                && !state.config.microsoft_client_secret.is_empty()
        }
    };
    if enabled && configured {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

fn client(state: &AppState, provider: Provider) -> AppResult<BasicClient> {
    let redirect = format!(
        "{}/{}",
        state.config.oauth_redirect_base.trim_end_matches('/'),
        provider.name()
    );
    let (client_id, client_secret, auth_url, token_url) = match provider {
        Provider::Google => (
            &state.config.google_client_id,
            &state.config.google_client_secret,
            "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
            "https://oauth2.googleapis.com/token".to_string(),
        ),
        Provider::Github => (
            &state.config.github_client_id,
            &state.config.github_client_secret,
            "https://github.com/login/oauth/authorize".to_string(),
            "https://github.com/login/oauth/access_token".to_string(),
        ),
        Provider::Microsoft => (
            &state.config.microsoft_client_id,
            &state.config.microsoft_client_secret,
            format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
                state.config.microsoft_tenant_id
            ),
            format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                state.config.microsoft_tenant_id
            ),
        ),
    };

    let mut client =
        BasicClient::new(
            ClientId::new(client_id.clone()),
            Some(ClientSecret::new(client_secret.clone())),
            AuthUrl::new(auth_url).map_err(|error| {
                AppError::Internal(anyhow::anyhow!("OAuth auth URL: {}", error))
            })?,
            Some(TokenUrl::new(token_url).map_err(|error| {
                AppError::Internal(anyhow::anyhow!("OAuth token URL: {}", error))
            })?),
        )
        .set_redirect_uri(RedirectUrl::new(redirect).map_err(|error| {
            AppError::Internal(anyhow::anyhow!("OAuth redirect URL: {}", error))
        })?);
    if matches!(provider, Provider::Microsoft) {
        client = client.set_auth_type(AuthType::RequestBody);
    }
    Ok(client)
}

fn scopes(provider: Provider) -> &'static [&'static str] {
    match provider {
        Provider::Google => &["openid", "email", "profile"],
        Provider::Github => &["read:user", "user:email"],
        Provider::Microsoft => &["openid", "email", "profile", "User.Read"],
    }
}

async fn consume_state(state: &AppState, csrf_state: &str) -> AppResult<PendingAuthorization> {
    let mut redis = state.redis.manager.clone();
    let value: Option<String> = redis::cmd("GETDEL")
        .arg(state_key(csrf_state))
        .query_async(&mut redis)
        .await?;
    let value = value.ok_or(AppError::InvalidToken)?;
    serde_json::from_str(&value).map_err(|_| AppError::InvalidToken)
}

fn state_key(csrf_state: &str) -> String {
    format!("oauth_state:{}", csrf_state)
}

async fn upsert_identity(
    state: &AppState,
    provider: Provider,
    profile: &ProviderProfile,
) -> AppResult<Uuid> {
    let oauth_repo = OAuthRepository::new(&state.db.pool);
    if let Some(account) = oauth_repo
        .find_by_provider(provider.name(), &profile.id)
        .await?
    {
        return Ok(account.user_id);
    }

    let user_repo = UserRepository::new(&state.db.pool);
    let user_id = if let Some(user) = user_repo.find_by_email(&profile.email).await? {
        user.id
    } else {
        let user_id = Uuid::new_v4();
        let mut tx = state.db.pool.begin().await?;
        UserRepository::create(
            &mut *tx,
            user_id,
            &profile.email,
            &profile.display_name,
            None,
            provider.name(),
        )
        .await?;
        UserRepository::set_email_verified(&mut *tx, user_id).await?;
        if let Some(role) = RoleRepository::new(&state.db.pool)
            .find_by_name("user")
            .await?
        {
            UserRepository::assign_role(&mut *tx, user_id, role.id, None).await?;
        }
        tx.commit().await?;
        user_id
    };

    OAuthRepository::upsert(
        &state.db.pool,
        user_id,
        provider.name(),
        &profile.id,
        Some(&profile.email),
        None,
        None,
        None,
    )
    .await?;
    Ok(user_id)
}

async fn fetch_profile(provider: Provider, access_token: &str) -> AppResult<ProviderProfile> {
    match provider {
        Provider::Google => fetch_google_profile(access_token).await,
        Provider::Github => fetch_github_profile(access_token).await,
        Provider::Microsoft => fetch_microsoft_profile(access_token).await,
    }
}

#[derive(Deserialize)]
struct GoogleProfile {
    sub: String,
    email: String,
    email_verified: bool,
    name: Option<String>,
}

async fn fetch_google_profile(access_token: &str) -> AppResult<ProviderProfile> {
    let profile = oauth_get::<GoogleProfile>(
        "https://openidconnect.googleapis.com/v1/userinfo",
        access_token,
    )
    .await?;
    if !profile.email_verified {
        return Err(AppError::OAuth(
            "Google account email is not verified".to_string(),
        ));
    }
    Ok(ProviderProfile {
        id: profile.sub,
        display_name: profile.name.unwrap_or_else(|| profile.email.clone()),
        email: profile.email,
    })
}

#[derive(Deserialize)]
struct GithubProfile {
    id: u64,
    login: String,
    name: Option<String>,
}

#[derive(Deserialize)]
struct GithubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

async fn fetch_github_profile(access_token: &str) -> AppResult<ProviderProfile> {
    let profile = oauth_get::<GithubProfile>("https://api.github.com/user", access_token).await?;
    let emails =
        oauth_get::<Vec<GithubEmail>>("https://api.github.com/user/emails", access_token).await?;
    let email = emails
        .iter()
        .find(|email| email.primary && email.verified)
        .or_else(|| emails.iter().find(|email| email.verified))
        .map(|email| email.email.clone())
        .ok_or_else(|| AppError::OAuth("GitHub did not provide a verified email".to_string()))?;
    Ok(ProviderProfile {
        id: profile.id.to_string(),
        display_name: profile.name.unwrap_or(profile.login),
        email,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MicrosoftProfile {
    id: String,
    display_name: String,
    mail: Option<String>,
    user_principal_name: String,
}

async fn fetch_microsoft_profile(access_token: &str) -> AppResult<ProviderProfile> {
    let profile = oauth_get::<MicrosoftProfile>(
        "https://graph.microsoft.com/v1.0/me?$select=id,displayName,mail,userPrincipalName",
        access_token,
    )
    .await?;
    Ok(ProviderProfile {
        id: profile.id,
        display_name: profile.display_name,
        email: profile.mail.unwrap_or(profile.user_principal_name),
    })
}

async fn oauth_get<T: for<'de> Deserialize<'de>>(url: &str, access_token: &str) -> AppResult<T> {
    reqwest::Client::new()
        .get(url)
        .bearer_auth(access_token)
        .header(reqwest::header::USER_AGENT, "AuthForge")
        .send()
        .await
        .map_err(|error| AppError::OAuth(format!("Profile request failed: {}", error)))?
        .error_for_status()
        .map_err(|error| AppError::OAuth(format!("Profile response failed: {}", error)))?
        .json()
        .await
        .map_err(|error| AppError::OAuth(format!("Profile payload invalid: {}", error)))
}

// ─── Account Linking ──────────────────────────────────────────────────────────

#[derive(Serialize)]
pub struct LinkedAccount {
    pub provider: String,
    pub provider_email: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_linked_accounts(
    state: &AppState,
    user_id: Uuid,
) -> AppResult<Vec<LinkedAccount>> {
    let accounts = OAuthRepository::new(&state.db.pool)
        .find_by_user(user_id)
        .await?;
    Ok(accounts
        .into_iter()
        .map(|a| LinkedAccount {
            provider: a.provider,
            provider_email: a.provider_email,
            created_at: a.created_at,
        })
        .collect())
}

pub async fn unlink_account(
    state: &AppState,
    user_id: Uuid,
    provider: &str,
) -> AppResult<()> {
    let accounts = OAuthRepository::new(&state.db.pool)
        .find_by_user(user_id)
        .await?;

    let account = accounts
        .iter()
        .find(|a| a.provider == provider)
        .ok_or_else(|| AppError::NotFound("linked account".to_string()))?;

    // Check if user has a password (can always unlink if they do)
    let user = UserRepository::new(&state.db.pool).get(user_id).await?;
    let has_password = user.password_hash.is_some();

    if !has_password && accounts.len() <= 1 {
        return Err(AppError::Conflict(
            "Cannot unlink the only authentication method".to_string(),
        ));
    }

    OAuthRepository::new(&state.db.pool)
        .delete_by_user_and_provider(user_id, &account.provider)
        .await?;

    Ok(())
}

pub async fn oauth_link_start(
    state: &AppState,
    _user_id: Uuid,
    provider_name: &str,
) -> AppResult<AuthorizationResponse> {
    // Reuse the same authorization URL flow but with a link-specific state
    authorization_url(state, provider_name).await
}
