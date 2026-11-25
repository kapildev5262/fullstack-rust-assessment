pub mod auth;
pub mod error;
pub mod models;
pub mod routes;
pub mod state;

use axum::{
    Json, Router,
    http::{HeaderValue, Method, header},
    routing::{get, post},
};
use serde_json::{Value, json};
use tower_http::cors::CorsLayer;

use state::AppState;

async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "task-manager-api"
    }))
}

pub fn build_app(state: AppState, frontend_url: &str) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            frontend_url
                .parse::<HeaderValue>()
                .expect("Invalid FRONTEND_URL"),
        )
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    Router::new()
        .route("/health", get(health))
        .route("/seed/users", post(routes::users::seed_users))
        .route("/auth/login", post(routes::auth::login))
        .route("/auth/verify-2fa", post(routes::auth::verify_2fa))
        .route("/dev/email-logs/latest", get(routes::auth::latest_email))
        .route(
            "/tasks",
            get(routes::tasks::list_tasks).post(routes::tasks::create_task),
        )
        .route("/tasks/assign", post(routes::tasks::assign_tasks))
        .route("/tasks/view-my-tasks", get(routes::tasks::view_my_tasks))
        .with_state(state)
        .layer(cors)
}
