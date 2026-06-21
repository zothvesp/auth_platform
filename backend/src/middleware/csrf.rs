use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use rand::Rng;

const CSRF_COOKIE_NAME: &str = "csrf_token";
const CSRF_HEADER_NAME: &str = "x-csrF-token";
const TOKEN_BYTES: usize = 32;
const MAX_AGE_SECS: u64 = 86400;

const SKIP_PREFIXES: &[&str] = &["/api/v1/auth", "/oauth/", "/saml/"];

fn should_skip(path: &str) -> bool {
    SKIP_PREFIXES.iter().any(|prefix| path.starts_with(prefix))
}

fn extract_cookie(headers: &axum::http::HeaderMap, name: &str) -> Option<String> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find(|pair| pair.trim().starts_with(name))
        .map(|pair| pair.trim()[name.len()..].trim().trim_start_matches('=').to_string())
}

fn generate_token() -> String {
    let mut rng = rand::thread_rng();
    (0..TOKEN_BYTES)
        .map(|_| format!("{:02x}", rng.gen::<u8>()))
        .collect()
}

pub async fn csrf_protection(request: Request<Body>, next: Next) -> Response {
    let path = request.uri().path().to_string();
    let method = request.method().clone();

    // Always skip OPTIONS preflight — CORS layer handles these
    if method == Method::OPTIONS {
        return next.run(request).await;
    }

    if should_skip(&path) {
        return next.run(request).await;
    }

    let cookie_token = extract_cookie(request.headers(), CSRF_COOKIE_NAME);

    if method != Method::GET && method != Method::HEAD {
        let header_token = request
            .headers()
            .get(CSRF_HEADER_NAME)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        if cookie_token.is_none()
            || header_token.is_none()
            || cookie_token != header_token
        {
            return (StatusCode::FORBIDDEN, "CSRF token validation failed").into_response();
        }
    }

    let mut response = next.run(request).await;

    if (method == Method::GET || method == Method::HEAD) && cookie_token.is_none() {
        let token = generate_token();
        let cookie = format!(
            "{CSRF_COOKIE_NAME}={token}; Path=/; SameSite=Lax; Max-Age={MAX_AGE_SECS}"
        );
        response
            .headers_mut()
            .append(header::SET_COOKIE, cookie.parse().unwrap());
    }

    response
}
