use crate::{error::AppResult, state::AppState};
use uuid::Uuid;

pub async fn verify_code(_state: &AppState, _user_id: Uuid, _code: &str) -> AppResult<()> {
    // TODO Phase 5: TOTP verification via totp-rs
    Ok(())
}

pub async fn setup(_state: &AppState, _user_id: Uuid) -> AppResult<(String, String)> {
    // TODO Phase 5: TOTP setup
    Ok((
        "qr_placeholder".to_string(),
        "secret_placeholder".to_string(),
    ))
}

pub async fn enable(_state: &AppState, _user_id: Uuid, _code: &str) -> AppResult<Vec<String>> {
    Ok(vec![])
}

pub async fn disable(_state: &AppState, _user_id: Uuid, _code: &str) -> AppResult<()> {
    Ok(())
}
