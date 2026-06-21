use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

const MAX_BODY_BYTES: usize = 1_048_576;

pub async fn validate_content_type(request: Request<Body>, next: Next) -> Response {
    let method = request.method().clone();

    if matches!(method, Method::POST | Method::PUT | Method::PATCH) {
        let content_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let content_length = request
            .headers()
            .get(header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(0);

        // Only enforce Content-Type when there IS a body
        if content_length > 0 {
            let ct = content_type.as_deref().unwrap_or("");
            if !ct.starts_with("application/json") {
                return (
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    axum::Json(serde_json::json!({
                        "code": "INVALID_CONTENT_TYPE",
                        "message": "Content-Type must be application/json",
                    })),
                )
                    .into_response();
            }
        }
    }

    next.run(request).await
}

pub fn body_limit_layer() -> tower_http::limit::RequestBodyLimitLayer {
    tower_http::limit::RequestBodyLimitLayer::new(MAX_BODY_BYTES)
}
