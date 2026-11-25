use std::env;

use backend::{build_app, state::AppState};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("Failed to load .env file");

    let database_url = env::var("DATABASE_URL")?;
    let jwt_secret = env::var("JWT_SECRET")?;

    if jwt_secret.len() < 32 {
        return Err("JWT_SECRET must contain at least 32 characters".into());
    }

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());

    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    let is_development = env::var("APP_ENV").as_deref() == Ok("development");

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    println!("PostgreSQL connected");

    sqlx::migrate!("./migrations").run(&pool).await?;

    println!("Database migrations completed");

    let state = AppState::new(pool, jwt_secret, is_development);

    let app = build_app(state, &frontend_url);

    let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;

    println!("Server running at http://{}:{}", host, port);

    axum::serve(listener, app).await?;

    Ok(())
}
