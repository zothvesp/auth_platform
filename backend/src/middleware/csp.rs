use axum::{body::Body, middleware::Next, http::Request, response::Response};

const CSP_POLICY: &str = "\
    default-src 'self'; \
    script-src 'self' 'unsafe-inline'; \
    style-src 'self' 'unsafe-inline'; \
    img-src 'self' data:; \
    connect-src 'self'; \
    frame-ancestors 'none'\
";

pub async fn csp_headers(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert("content-security-policy", CSP_POLICY.parse().unwrap());
    response
}
