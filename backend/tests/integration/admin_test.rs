use super::helpers;

#[tokio::test]
async fn test_list_users_requires_permission() {
    let server = helpers::start_server().await;

    let resp = helpers::client()
        .get(format!("{}/api/v1/admin/users", server.base_url))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status(),
        401,
        "admin endpoint without auth should return 401"
    );
}

#[tokio::test]
async fn test_create_user() {
    let server = helpers::start_server().await;
    let (_, _, admin_token) = helpers::create_admin_user(&server.base_url).await;
    let csrf = helpers::get_csrf_token(&server.base_url).await;

    let new_email = format!("newuser-{}@integ.test", uuid::Uuid::new_v4());
    let resp = helpers::client()
        .post(format!("{}/api/v1/admin/users", server.base_url))
        .header("authorization", format!("Bearer {}", admin_token))
        .header("content-type", "application/json")
        .header("x-csrf-token", &csrf)
        .header("cookie", format!("csrf_token={}", csrf))
        .json(&serde_json::json!({
            "email": new_email,
            "displayName": "Created By Admin",
            "password": "Adm1n!Test#2024",
        }))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .expect("request failed");

    assert_eq!(
        resp.status(),
        201,
        "admin create user should return 201"
    );

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["email"].as_str().unwrap(), new_email);
    assert_eq!(body["displayName"].as_str().unwrap(), "Created By Admin");
}
