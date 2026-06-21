use axum::{
    body::Body,
    extract::State,
    http::{header, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use tracing::warn;

use crate::state::AppState;

struct RateLimitRule {
    max_requests: u32,
    window_secs: usize,
}

fn rate_limit_for_path(path: &str) -> Option<RateLimitRule> {
    match path {
        "/api/v1/auth/login" => Some(RateLimitRule {
            max_requests: 5,
            window_secs: 900, // 5 attempts per 15 minutes
        }),
        "/api/v1/auth/register" => Some(RateLimitRule {
            max_requests: 5,
            window_secs: 900,
        }),
        "/api/v1/auth/forgot-password" => Some(RateLimitRule {
            max_requests: 3,
            window_secs: 900,
        }),
        _ => None,
    }
}

pub async fn ip_rate_limit(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if *request.method() == Method::GET {
        return next.run(request).await;
    }

    let path = request.uri().path();
    if path == "/health" {
        return next.run(request).await;
    }

    let rule = match rate_limit_for_path(path) {
        Some(r) => r,
        None => return next.run(request).await,
    };

    let ip = crate::utils::extract_ip(request.headers());
    let key = format!("rate_limit:{}:{}", path, ip);

    let mut conn = state.redis.manager.clone();
    let count: u32 = match conn.incr::<_, _, u32>(&key, 1).await {
        Ok(c) => c,
        Err(e) => {
            warn!("Rate limit Redis error (fail-open): {}", e);
            return next.run(request).await;
        }
    };

    if count == 1 {
        let _: () = conn
            .expire(&key, rule.window_secs)
            .await
            .ok()
            .unwrap_or(());
    }

    if count > rule.max_requests {
        let retry_after: u64 = conn
            .ttl::<_, i64>(&key)
            .await
            .map(|t| t.max(1) as u64)
            .unwrap_or(rule.window_secs as u64);

        return (
            StatusCode::TOO_MANY_REQUESTS,
            [
                (header::RETRY_AFTER, retry_after.to_string()),
                (header::CONTENT_TYPE, "application/json".to_string()),
            ],
            serde_json::json!({
                "code": "RATE_LIMITED",
                "message": "Too many requests. Please try again later.",
            })
            .to_string(),
        )
            .into_response();
    }

    next.run(request).await
}
