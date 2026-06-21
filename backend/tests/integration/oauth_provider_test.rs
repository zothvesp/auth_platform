use super::helpers;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_oidc_discovery() {
    let server = helpers::start_server().await;

    let resp = helpers::client()
        .get(format!(
            "{}/.well-known/openid-configuration",
            server.base_url
        ))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200, "OIDC discovery should return 200");

    let body: serde_json::Value = resp.json().await.unwrap();

    assert!(
        body["issuer"].is_string(),
        "response must include issuer"
    );
    assert!(
        body["authorization_endpoint"].is_string(),
        "response must include authorization_endpoint"
    );
    assert!(
        body["token_endpoint"].is_string(),
        "response must include token_endpoint"
    );
    assert!(
        body["jwks_uri"].is_string(),
        "response must include jwks_uri"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_jwks() {
    let server = helpers::start_server().await;

    let resp = helpers::client()
        .get(format!("{}/oauth/jwks", server.base_url))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 200, "JWKS endpoint should return 200");

    let body: serde_json::Value = resp.json().await.unwrap();
    let keys = body["keys"]
        .as_array()
        .expect("response must include keys array");

    assert!(!keys.is_empty(), "keys array should not be empty");

    let first_key = &keys[0];
    assert!(
        first_key["kty"].is_string(),
        "key must have kty (key type)"
    );
    assert!(first_key["kid"].is_string(), "key must have kid (key id)");
}
