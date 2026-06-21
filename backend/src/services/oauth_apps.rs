//! OAuth Apps admin service — CRUD for OAuth application registrations.
//! No SQL here. All DB access goes through repositories.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    models::OAuthApp,
    repositories::{base::BaseRepository, OAuthAppRepository},
    state::AppState,
};

// ─── DTOs ─────────────────────────────────────────────────────────────────────

#[derive(serde::Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateOAuthAppInput {
    #[validate(length(min = 1, max = 255, message = "Name is required"))]
    pub name: String,
    pub description: Option<String>,
    #[validate(length(min = 1, message = "At least one redirect URI is required"))]
    pub redirect_uris: Vec<String>,
    pub allowed_grants: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub pkce_required: Option<bool>,
    pub logo_url: Option<String>,
}

#[derive(serde::Deserialize, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOAuthAppInput {
    #[validate(length(min = 1, max = 255, message = "Name is required"))]
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub redirect_uris: Option<Vec<String>>,
    pub allowed_grants: Option<Vec<String>>,
    pub allowed_scopes: Option<Vec<String>>,
    pub pkce_required: Option<bool>,
    pub logo_url: Option<Option<String>>,
    pub is_active: Option<bool>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthAppResponse {
    pub id: Uuid,
    pub client_id: String,
    pub name: String,
    pub description: Option<String>,
    pub redirect_uris: Vec<String>,
    pub allowed_grants: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub pkce_required: bool,
    pub logo_url: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOAuthAppResponse {
    pub id: Uuid,
    pub client_id: String,
    pub client_secret: String,
    pub name: String,
    pub description: Option<String>,
    pub redirect_uris: Vec<String>,
    pub allowed_grants: Vec<String>,
    pub allowed_scopes: Vec<String>,
    pub pkce_required: bool,
    pub logo_url: Option<String>,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

// ─── Business Logic ───────────────────────────────────────────────────────────

pub async fn list_apps(state: &AppState) -> AppResult<Vec<OAuthAppResponse>> {
    let apps = OAuthAppRepository::new(&state.db.pool).find_all().await?;
    Ok(apps.into_iter().map(OAuthAppResponse::from).collect())
}

pub async fn get_app(state: &AppState, id: Uuid) -> AppResult<OAuthAppResponse> {
    let app = OAuthAppRepository::new(&state.db.pool).get(id).await?;
    Ok(OAuthAppResponse::from(app))
}

pub async fn create_app(
    state: &AppState,
    input: &CreateOAuthAppInput,
) -> AppResult<CreateOAuthAppResponse> {
    let repo = OAuthAppRepository::new(&state.db.pool);

    let client_id = generate_client_id();
    let client_secret = generate_client_secret();
    let client_secret_hash = hash_secret(&client_secret);

    let app = repo
        .create(
            &client_id,
            &client_secret_hash,
            &input.name,
            input.description.as_deref(),
            &input.redirect_uris,
            &input.allowed_grants,
            &input.allowed_scopes,
            input.pkce_required.unwrap_or(false),
            input.logo_url.as_deref(),
        )
        .await?;

    Ok(CreateOAuthAppResponse {
        id: app.id,
        client_id: app.client_id,
        client_secret,
        name: app.name,
        description: app.description,
        redirect_uris: app.redirect_uris,
        allowed_grants: app.allowed_grants,
        allowed_scopes: app.allowed_scopes,
        pkce_required: app.pkce_required,
        logo_url: app.logo_url,
        is_active: app.is_active,
        created_at: app.created_at,
        updated_at: app.updated_at,
    })
}

pub async fn update_app(
    state: &AppState,
    id: Uuid,
    input: &UpdateOAuthAppInput,
) -> AppResult<OAuthAppResponse> {
    let repo = OAuthAppRepository::new(&state.db.pool);
    let existing = repo.get(id).await?;

    let name = input.name.as_deref().unwrap_or(&existing.name);
    let description = match &input.description {
        Some(d) => d.as_deref(),
        None => existing.description.as_deref(),
    };
    let redirect_uris = input.redirect_uris.as_deref().unwrap_or(&existing.redirect_uris);
    let allowed_grants = input.allowed_grants.as_deref().unwrap_or(&existing.allowed_grants);
    let allowed_scopes = input.allowed_scopes.as_deref().unwrap_or(&existing.allowed_scopes);
    let pkce_required = input.pkce_required.unwrap_or(existing.pkce_required);
    let logo_url = match &input.logo_url {
        Some(l) => l.as_deref(),
        None => existing.logo_url.as_deref(),
    };
    let is_active = input.is_active.unwrap_or(existing.is_active);

    repo.update(
        id,
        name,
        description,
        redirect_uris,
        allowed_grants,
        allowed_scopes,
        pkce_required,
        logo_url,
        is_active,
    )
    .await?;

    let updated = repo.get(id).await?;
    Ok(OAuthAppResponse::from(updated))
}

pub async fn delete_app(state: &AppState, id: Uuid) -> AppResult<()> {
    OAuthAppRepository::new(&state.db.pool)
        .find_by_id(id)
        .await?
        .ok_or_else(|| AppError::NotFound("OAuth app".to_string()))?;

    OAuthAppRepository::new(&state.db.pool).delete(id).await
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn generate_client_id() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}

fn generate_client_secret() -> String {
    use rand::Rng;
    let bytes: Vec<u8> = rand::thread_rng()
        .sample_iter(&rand::distributions::Standard)
        .take(48)
        .collect();
    URL_SAFE_NO_PAD.encode(&bytes)
}

fn hash_secret(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    base64::engine::general_purpose::STANDARD.encode(hasher.finalize())
}

// ─── Conversions ──────────────────────────────────────────────────────────────

impl From<OAuthApp> for OAuthAppResponse {
    fn from(app: OAuthApp) -> Self {
        Self {
            id: app.id,
            client_id: app.client_id,
            name: app.name,
            description: app.description,
            redirect_uris: app.redirect_uris,
            allowed_grants: app.allowed_grants,
            allowed_scopes: app.allowed_scopes,
            pkce_required: app.pkce_required,
            logo_url: app.logo_url,
            is_active: app.is_active,
            created_at: app.created_at,
            updated_at: app.updated_at,
        }
    }
}
