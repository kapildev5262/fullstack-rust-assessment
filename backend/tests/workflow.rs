use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use backend::{build_app, state::AppState};
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

fn test_app(pool: PgPool) -> Router {
    let state = AppState::new(
        pool,
        "integration-test-secret-at-least-32-characters".into(),
        true,
    );

    build_app(state, "http://localhost:5173")
}

async fn request(
    app: &Router,
    method: Method,
    path: &str,
    body: Option<Value>,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path);

    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    if let Some(token) = token {
        builder = builder.header("Authorization", format!("Bearer {token}"));
    }

    let body = match body {
        Some(value) => Body::from(value.to_string()),
        None => Body::empty(),
    };

    let response = app
        .clone()
        .oneshot(builder.body(body).unwrap())
        .await
        .unwrap();

    let status = response.status();

    let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();

    let data = serde_json::from_slice(&bytes).unwrap_or(Value::Null);

    (status, data)
}

async fn seed(app: &Router) {
    let (status, body) = request(app, Method::POST, "/seed/users", None, None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["users"].as_array().unwrap().len(), 2);
}

async fn start_login(app: &Router, email: &str, password: &str) -> (String, String) {
    let (status, body) = request(
        app,
        Method::POST,
        "/auth/login",
        Some(json!({
            "email": email,
            "password": password
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body.get("access_token").is_none());
    assert!(body.get("token").is_none());

    let challenge = body["login_challenge_id"].as_str().unwrap().to_string();

    let (status, log) = request(app, Method::GET, "/dev/email-logs/latest", None, None).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(log["login_challenge_id"], challenge);
    assert_eq!(log["email"], email);

    let code = log["code"].as_str().unwrap().to_string();

    (challenge, code)
}

async fn verify(app: &Router, challenge: &str, code: &str) -> (StatusCode, Value) {
    request(
        app,
        Method::POST,
        "/auth/verify-2fa",
        Some(json!({
            "login_challenge_id": challenge,
            "code": code
        })),
        None,
    )
    .await
}

async fn login(app: &Router, email: &str, password: &str) -> String {
    let (challenge, code) = start_login(app, email, password).await;

    let (status, body) = verify(app, &challenge, &code).await;

    assert_eq!(status, StatusCode::OK);

    body["access_token"].as_str().unwrap().to_string()
}

// Test 1: Seed users without creating duplicates.
#[sqlx::test(migrations = "./migrations")]
async fn seed_users_is_idempotent(pool: PgPool) {
    let app = test_app(pool.clone());

    seed(&app).await;
    seed(&app).await;

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(count, 2);
}

// Test 2: Invalid credentials are rejected.
#[sqlx::test(migrations = "./migrations")]
async fn invalid_credentials_rejected(pool: PgPool) {
    let app = test_app(pool);

    seed(&app).await;

    let (status, body) = request(
        &app,
        Method::POST,
        "/auth/login",
        Some(json!({
            "email": "admin@example.com",
            "password": "incorrect-password"
        })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(body.get("access_token").is_none());
}

// Test 3: Incorrect code, successful verification and reuse.
#[sqlx::test(migrations = "./migrations")]
async fn verification_code_lifecycle(pool: PgPool) {
    let app = test_app(pool);

    seed(&app).await;

    let (challenge, code) = start_login(&app, "admin@example.com", "Admin@123").await;

    let wrong_code = if code == "000000" { "111111" } else { "000000" };

    let (status, body) = verify(&app, &challenge, wrong_code).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body.get("access_token").is_none());

    let (status, body) = verify(&app, &challenge, &code).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["access_token"].is_string());
    assert_eq!(body["user"]["role"], "admin");

    let (status, body) = verify(&app, &challenge, &code).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Verification code already used");
}

// Test 4: Expired verification code is rejected.
#[sqlx::test(migrations = "./migrations")]
async fn expired_code_rejected(pool: PgPool) {
    let app = test_app(pool.clone());

    seed(&app).await;

    let (challenge, code) = start_login(&app, "admin@example.com", "Admin@123").await;

    sqlx::query(
        "UPDATE login_challenges
         SET expires_at = NOW() - INTERVAL '1 minute'
         WHERE id = $1",
    )
    .bind(Uuid::parse_str(&challenge).unwrap())
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = verify(&app, &challenge, &code).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "Verification code expired");
}

// Test 5: Unauthenticated requests are rejected.
#[sqlx::test(migrations = "./migrations")]
async fn missing_and_invalid_jwt_rejected(pool: PgPool) {
    let app = test_app(pool);

    for token in [None, Some("invalid-jwt")] {
        let (status, _) = request(&app, Method::GET, "/tasks/view-my-tasks", None, token).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
}

// Test 6: Complete assessment workflow.
#[sqlx::test(migrations = "./migrations")]
async fn complete_task_management_workflow(pool: PgPool) {
    let app = test_app(pool.clone());

    seed(&app).await;

    let admin_token = login(&app, "admin@example.com", "Admin@123").await;

    let mut task_ids = Vec::new();

    let priorities = ["high", "medium", "low", "medium", "high"];

    for (index, priority) in priorities.iter().enumerate() {
        let (status, body) = request(
            &app,
            Method::POST,
            "/tasks",
            Some(json!({
                "title": format!("Task {}", index + 1),
                "description": "Integration test task",
                "priority": priority
            })),
            Some(&admin_token),
        )
        .await;

        assert_eq!(status, StatusCode::CREATED);

        task_ids.push(body["id"].as_str().unwrap().to_string());
    }

    let (status, body) = request(&app, Method::GET, "/tasks", None, Some(&admin_token)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 5);

    let assignment = json!({
        "task_ids": &task_ids[..3],
        "assigned_to_email": "jamesbond@example.com"
    });

    let (status, body) = request(
        &app,
        Method::POST,
        "/tasks/assign",
        Some(assignment.clone()),
        Some(&admin_token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["assigned_count"], 3);

    let staff_token = login(&app, "jamesbond@example.com", "James@123").await;

    // Staff cannot create tasks.
    let (status, _) = request(
        &app,
        Method::POST,
        "/tasks",
        Some(json!({
            "title": "Unauthorized task",
            "priority": "high"
        })),
        Some(&staff_token),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    // Staff cannot assign tasks.
    let (status, _) = request(
        &app,
        Method::POST,
        "/tasks/assign",
        Some(assignment.clone()),
        Some(&staff_token),
    )
    .await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    // Staff cannot list all tasks.
    let (status, _) = request(&app, Method::GET, "/tasks", None, Some(&staff_token)).await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    // First request must miss the cache.
    let (status, first) = request(
        &app,
        Method::GET,
        "/tasks/view-my-tasks",
        None,
        Some(&staff_token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(first["cache"]["hit"], false);
    assert_eq!(first["summary"]["total_assigned_tasks"], 3);
    assert_eq!(first["user"]["role"], "staff");

    let tasks = first["tasks"].as_array().unwrap();

    assert_eq!(tasks.len(), 3);

    for task in tasks {
        assert_eq!(task["assigned_to"], "jamesbond@example.com");

        assert!(task_ids[..3].contains(&task["id"].as_str().unwrap().to_string()));
    }

    // Second request must hit the cache.
    let (status, second) = request(
        &app,
        Method::GET,
        "/tasks/view-my-tasks",
        None,
        Some(&staff_token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(second["cache"]["hit"], true);
    assert_eq!(first["tasks"], second["tasks"]);

    // Admin must not receive staff's cached tasks.
    let (status, admin_tasks) = request(
        &app,
        Method::GET,
        "/tasks/view-my-tasks",
        None,
        Some(&admin_token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(admin_tasks["summary"]["total_assigned_tasks"], 0);

    // Reassign tasks to invalidate staff cache.
    let (status, _) = request(
        &app,
        Method::POST,
        "/tasks/assign",
        Some(assignment),
        Some(&admin_token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);

    let (status, after_assignment) = request(
        &app,
        Method::GET,
        "/tasks/view-my-tasks",
        None,
        Some(&staff_token),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(after_assignment["cache"]["hit"], false);
    assert_eq!(after_assignment["summary"]["total_assigned_tasks"], 3);

    // Unauthorized creation must not change task count.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(count, 5);
}

// Test 7: Verification attempt limit.
#[sqlx::test(migrations = "./migrations")]
async fn verification_attempt_limit(pool: PgPool) {
    let app = test_app(pool);

    seed(&app).await;

    let (challenge, code) = start_login(&app, "admin@example.com", "Admin@123").await;

    let wrong_code = if code == "000000" { "111111" } else { "000000" };

    for _ in 0..5 {
        let (status, _) = verify(&app, &challenge, wrong_code).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
    }

    let (status, _) = verify(&app, &challenge, &code).await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}
