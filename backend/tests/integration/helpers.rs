use tokio::sync::oneshot;

use authforge_backend::{
    build_router,
    config::AppConfig,
    db::{Database, RedisPool},
    state::AppState,
    vault,
};

pub struct TestServer {
    pub base_url: String,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

pub async fn start_server() -> TestServer {
    dotenvy::dotenv().ok();
    let config = AppConfig::from_env().expect("Failed to load AppConfig from env");

    let db = Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres");
    db.run_migrations()
        .await
        .expect("Migrations failed");

    let redis = RedisPool::connect_or_noop(&config.redis_url).await;
    let vault_impl = vault::create_vault(db.pool.clone());
    let state = AppState::new(db, redis, config.clone(), vault_impl);
    authforge_backend::services::rbac::seed_defaults(&state)
        .await
        .expect("RBAC seed failed");

    let app = build_router(state, &config);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let base_url = format!("http://127.0.0.1:{}", port);

    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
            .expect("Server failed");
    });

    for _ in 0..50 {
        if tokio::net::TcpStream::connect(format!("127.0.0.1:{}", port))
            .await
            .is_ok()
        {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    TestServer {
        base_url,
        shutdown_tx: Some(shutdown_tx),
    }
}

pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .unwrap()
}

pub async fn register_user(base_url: &str, email: &str, password: &str) -> String {
    let resp = client()
        .post(format!("{}/api/v1/auth/register", base_url))
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "email": email,
            "displayName": "Test User",
            "password": password,
        }))
        .send()
        .await
        .expect("register request failed");

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(status.is_success(), "Register failed ({}): {}", status, body);

    body["tokens"]["accessToken"]
        .as_str()
        .expect("missing accessToken")
        .to_string()
}

pub async fn login(base_url: &str, email: &str, password: &str) -> String {
    let resp = client()
        .post(format!("{}/api/v1/auth/login", base_url))
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "email": email,
            "password": password,
        }))
        .send()
        .await
        .expect("login request failed");

    let status = resp.status();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(status.is_success(), "Login failed ({}): {}", status, body);

    body["tokens"]["accessToken"]
        .as_str()
        .expect("missing accessToken")
        .to_string()
}

pub async fn get_csrf_token(base_url: &str) -> String {
    let resp = client()
        .get(format!("{}/health", base_url))
        .send()
        .await
        .expect("health request failed");

    for cookie in resp.headers().get_all("set-cookie").iter() {
        let cookie_str = cookie.to_str().unwrap_or("");
        if cookie_str.starts_with("csrf_token=") {
            return cookie_str
                .split(';')
                .next()
                .unwrap()
                .trim_start_matches("csrf_token=")
                .to_string();
        }
    }
    panic!("No csrf_token cookie found in response");
}

pub async fn create_admin_user(base_url: &str) -> (String, String, String) {
    let config = AppConfig::from_env().expect("Failed to load config");
    let db = Database::connect(&config.database_url)
        .await
        .expect("Failed to connect to DB");

    let uid = uuid::Uuid::new_v4();
    let email = format!("admin-{}@integ.test", uid);
    let password = String::from("Adm1n!Test#2024");

    let _token = register_user(base_url, &email, &password).await;

    let user_row = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM users WHERE LOWER(email) = LOWER($1)",
    )
    .bind(&email)
    .fetch_one(&db.pool)
    .await
    .expect("user not found after register");

    let role_id = sqlx::query_scalar::<_, uuid::Uuid>(
        "SELECT id FROM roles WHERE name = 'super_admin'",
    )
    .fetch_one(&db.pool)
    .await
    .expect("super_admin role not found — run seed_defaults first");

    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(user_row)
        .bind(role_id)
        .execute(&db.pool)
        .await
        .expect("failed to assign super_admin role");

    let admin_token = login(base_url, &email, &password).await;

    (email, password.to_string(), admin_token)
}
