//! Email dispatch service.
//! Uses token repositories to store tokens, then sends via SMTP.
//! No sqlx calls — all DB through repositories.

use chrono::Utc;
use rand::Rng;
use tracing::info;
use uuid::Uuid;

use crate::{
    error::AppResult,
    repositories::{
        base::BaseRepository, EmailTokenRepository, PasswordResetTokenRepository, UserRepository,
    },
    services::auth::hash_token,
    state::AppState,
};

pub async fn send_verification(state: &AppState, user_id: Uuid) -> AppResult<()> {
    let token = random_token();
    let token_hash = hash_token(&token);
    let expiry_hrs = crate::services::config::email_verification_expiry_hrs(state).await;
    let expires_at = Utc::now() + chrono::Duration::hours(expiry_hrs);
    let pool = &state.db.pool;

    EmailTokenRepository::delete_by_user(pool, user_id).await?;
    EmailTokenRepository::create(pool, user_id, &token_hash, expires_at).await?;

    let email = UserRepository::new(pool)
        .get(user_id)
        .await
        .map(|u| u.email)?;
    let url = format!("{}/verify-email?token={}", state.config.app_base_url, token);

    dispatch(
        state,
        &email,
        "Verify your AuthForge account",
        &format!("Verify your email: {}", url),
        &format!(r#"<h2>Verify your email</h2>
            <a href="{url}" style="background:#6366f1;color:#fff;padding:12px 24px;border-radius:8px;text-decoration:none">
              Verify Email
            </a>
            <p style="color:#64748b;font-size:14px">Expires in 24 hours.</p>"#),
    ).await
}

pub async fn send_password_reset(state: &AppState, user_id: Uuid) -> AppResult<()> {
    let token = random_token();
    let token_hash = hash_token(&token);
    let expiry_mins = crate::services::config::password_reset_expiry_mins(state).await;
    let expires_at = Utc::now() + chrono::Duration::minutes(expiry_mins);
    let pool = &state.db.pool;

    PasswordResetTokenRepository::delete_by_user(pool, user_id).await?;
    PasswordResetTokenRepository::create(pool, user_id, &token_hash, expires_at).await?;

    let email = UserRepository::new(pool)
        .get(user_id)
        .await
        .map(|u| u.email)?;
    let url = format!(
        "{}/reset-password?token={}",
        state.config.app_base_url, token
    );

    dispatch(
        state,
        &email,
        "Reset your AuthForge password",
        &format!("Reset link (15 min): {}", url),
        &format!(r#"<h2>Reset your password</h2>
            <a href="{url}" style="background:#ef4444;color:#fff;padding:12px 24px;border-radius:8px;text-decoration:none">
              Reset Password
            </a>
            <p style="color:#64748b;font-size:14px">Expires in 15 minutes.</p>"#),
    ).await
}

// ─── Internal helpers ─────────────────────────────────────────────────────────

fn random_token() -> String {
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

async fn dispatch(
    state: &AppState,
    to: &str,
    subject: &str,
    text: &str,
    html: &str,
) -> AppResult<()> {
    use crate::error::AppError;
    use lettre::{
        message::header::ContentType, transport::smtp::authentication::Credentials,
        AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
    };

    let message = Message::builder()
        .from(
            state
                .config
                .smtp_from
                .parse()
                .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?,
        )
        .to(to
            .parse()
            .map_err(|e| AppError::Internal(anyhow::anyhow!("{}", e)))?)
        .subject(subject)
        .multipart(
            lettre::message::MultiPart::alternative()
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(text.to_string()),
                )
                .singlepart(
                    lettre::message::SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(html.to_string()),
                ),
        )
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Email build: {}", e)))?;

    AsyncSmtpTransport::<Tokio1Executor>::relay(&state.config.smtp_host)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("SMTP relay: {}", e)))?
        .credentials(Credentials::new(
            state.config.smtp_username.clone(),
            state.config.smtp_password.clone(),
        ))
        .port(state.config.smtp_port)
        .build()
        .send(message)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Email send: {}", e)))?;

    info!("Email sent to {}: {}", to, subject);
    Ok(())
}
