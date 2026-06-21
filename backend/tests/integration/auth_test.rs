use uuid::Uuid;

use super::helpers;

#[tokio::test]
async fn test_register_and_login() {
    let server = helpers::start_server().await;
    let email = format!("integ-register-{}@test.local", Uuid::new_v4());
    let password = "Str0ng!Pass#2024";

    let token = helpers::register_user(&server.base_url, &email, password).await;
    assert!(!token.is_empty(), "access token should not be empty");
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let server = helpers::start_server().await;
    let email = format!("integ-invalid-{}@test.local", Uuid::new_v4());
    let password = "Str0ng!Pass#2024";

    helpers::register_user(&server.base_url, &email, password).await;

    let resp = helpers::client()
        .post(format!("{}/api/v1/auth/login", server.base_url))
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "email": email,
            "password": "wrong-password",
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 401, "wrong password should return 401");
}

#[tokio::test]
async fn test_register_duplicate_email() {
    let server = helpers::start_server().await;
    let email = format!("integ-dup-{}@test.local", Uuid::new_v4());
    let password = "Str0ng!Pass#2024";

    helpers::register_user(&server.base_url, &email, password).await;

    let resp = helpers::client()
        .post(format!("{}/api/v1/auth/register", server.base_url))
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "email": email,
            "displayName": "Duplicate User",
            "password": password,
        }))
        .send()
        .await
        .expect("request failed");

    assert_eq!(resp.status(), 409, "duplicate email should return 409");
}
