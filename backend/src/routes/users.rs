use axum::{Json, extract::State};
use serde_json::{Value, json};

use crate::{
    auth::hash_password,
    error::AppError,
    models::{User, UserInfo},
    state::AppState,
};

pub async fn seed_users(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    if !state.is_development {
        return Err(AppError::NotFound("Endpoint unavailable".into()));
    }

    let users = [
        ("Admin", "admin@example.com", "Admin@123", "admin"),
        ("James Bond", "jamesbond@example.com", "James@123", "staff"),
    ];

    let mut seeded = Vec::new();
    let mut tx = state.db.begin().await?;

    for (name, email, password, role) in users {
        let hashed = hash_password(password.to_string()).await?;

        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users
                (full_name, email, hashed_password, role)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (email) DO UPDATE
             SET full_name = EXCLUDED.full_name,
                 hashed_password = EXCLUDED.hashed_password,
                 role = EXCLUDED.role,
                 updated_at = NOW()
             RETURNING *",
        )
        .bind(name)
        .bind(email)
        .bind(hashed)
        .bind(role)
        .fetch_one(&mut *tx)
        .await?;

        seeded.push(UserInfo::from(&user));
    }

    tx.commit().await?;

    Ok(Json(json!({
        "message": "Users seeded successfully",
        "users": seeded
    })))
}
