use rand::{distributions::Alphanumeric, Rng};
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

use crate::{
    error::{AppError, AppResult},
    repositories::{base::BaseRepository, MfaRepository, UserRepository},
    services::auth::hash_token,
    state::AppState,
};

const BACKUP_CODE_COUNT: usize = 10;

pub async fn verify_code(state: &AppState, user_id: Uuid, code: &str) -> AppResult<()> {
    let normalized = normalize_code(code);
    let user = UserRepository::new(&state.db.pool).get(user_id).await?;
    let secret = user.mfa_secret.ok_or(AppError::MfaNotEnabled)?;

    if normalized.len() == 6
        && totp(&secret, &user.email)?
            .check_current(&normalized)
            .unwrap_or(false)
    {
        return Ok(());
    }

    if MfaRepository::new(&state.db.pool)
        .consume_backup_code(user_id, &hash_token(&normalized))
        .await?
    {
        return Ok(());
    }

    Err(AppError::InvalidMfaCode)
}

pub async fn setup(state: &AppState, user_id: Uuid) -> AppResult<(String, String)> {
    let user = UserRepository::new(&state.db.pool).get(user_id).await?;
    if user.mfa_enabled {
        return Err(AppError::Conflict("MFA is already enabled".to_string()));
    }

    let secret = Secret::generate_secret().to_encoded().to_string();
    let totp = totp(&secret, &user.email)?;
    let enrollment_uri = totp.get_url();

    UserRepository::set_mfa(&state.db.pool, user_id, false, Some(&secret)).await?;
    Ok((enrollment_uri, secret))
}

pub async fn enable(state: &AppState, user_id: Uuid, code: &str) -> AppResult<Vec<String>> {
    let user = UserRepository::new(&state.db.pool).get(user_id).await?;
    if user.mfa_enabled {
        return Err(AppError::Conflict("MFA is already enabled".to_string()));
    }

    let secret = user.mfa_secret.ok_or(AppError::MfaNotEnabled)?;
    let normalized = normalize_code(code);
    if normalized.len() != 6
        || !totp(&secret, &user.email)?
            .check_current(&normalized)
            .unwrap_or(false)
    {
        return Err(AppError::InvalidMfaCode);
    }

    let codes = generate_backup_codes();
    let hashes = codes
        .iter()
        .map(|backup_code| hash_token(&normalize_code(backup_code)))
        .collect::<Vec<_>>();

    let mut tx = state.db.pool.begin().await?;
    UserRepository::set_mfa(&mut *tx, user_id, true, Some(&secret)).await?;
    MfaRepository::replace_backup_codes(&mut tx, user_id, &hashes).await?;
    tx.commit().await?;

    Ok(codes)
}

pub async fn disable(state: &AppState, user_id: Uuid, code: &str) -> AppResult<()> {
    let user = UserRepository::new(&state.db.pool).get(user_id).await?;
    if !user.mfa_enabled {
        return Err(AppError::MfaNotEnabled);
    }

    verify_code(state, user_id, code).await?;

    let mut tx = state.db.pool.begin().await?;
    UserRepository::set_mfa(&mut *tx, user_id, false, None).await?;
    MfaRepository::delete_backup_codes(&mut *tx, user_id).await?;
    tx.commit().await?;
    Ok(())
}

fn totp(secret: &str, account_name: &str) -> AppResult<TOTP> {
    let bytes = Secret::Encoded(secret.to_string())
        .to_bytes()
        .map_err(|error| AppError::Internal(anyhow::anyhow!("MFA secret: {}", error)))?;

    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        bytes,
        Some("AuthForge".to_string()),
        account_name.to_string(),
    )
    .map_err(|error| AppError::Internal(anyhow::anyhow!("MFA configuration: {}", error)))
}

fn normalize_code(code: &str) -> String {
    code.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_uppercase)
        .collect()
}

fn generate_backup_codes() -> Vec<String> {
    (0..BACKUP_CODE_COUNT)
        .map(|_| {
            let raw = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(12)
                .map(char::from)
                .collect::<String>()
                .to_uppercase();
            format!("{}-{}-{}", &raw[0..4], &raw[4..8], &raw[8..12])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{generate_backup_codes, normalize_code};

    #[test]
    fn backup_codes_have_expected_format() {
        let codes = generate_backup_codes();
        assert_eq!(codes.len(), 10);
        assert!(codes.iter().all(|code| code.len() == 14));
    }

    #[test]
    fn codes_are_normalized_for_comparison() {
        assert_eq!(normalize_code("ab12-cd34-ef56"), "AB12CD34EF56");
    }
}
