use axum::{Json, extract::State};
use chrono::{Duration, Utc};
use rand::{Rng, rngs::OsRng};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    auth::{create_token, hash_password, verify_password},
    error::AppError,
    models::{EmailLog, LoginChallenge, User, UserInfo},
    state::AppState,
};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub login_challenge_id: Uuid,
    pub code: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, AppError> {
    let email = body.email.trim().to_lowercase();

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
        .bind(email)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".into()))?;

    let valid = verify_password(body.password, user.hashed_password.clone()).await?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid email or password".into()));
    }

    // Generate a cryptographically random six-digit code.
    let code = format!("{:06}", OsRng.gen_range(0..1_000_000));

    let code_hash = hash_password(code.clone()).await?;
    let challenge_id = Uuid::new_v4();
    let expires_at = Utc::now() + Duration::minutes(5);

    sqlx::query(
        "INSERT INTO login_challenges
            (id, user_id, code_hash, expires_at)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(challenge_id)
    .bind(user.id)
    .bind(code_hash)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    if state.is_development {
        let email_log = EmailLog {
            email: user.email.clone(),
            login_challenge_id: challenge_id,
            code: code.clone(),
            created_at: Utc::now(),
        };

        *state.email_log.write().await = Some(email_log);

        println!(
            "[DEV EMAIL] To: {} | Challenge: {} | Code: {}",
            user.email, challenge_id, code
        );
    }

    Ok(Json(json!({
        "login_challenge_id": challenge_id,
        "message": "Verification code sent",
        "expires_in": 300
    })))
}

pub async fn latest_email(State(state): State<AppState>) -> Result<Json<EmailLog>, AppError> {
    if !state.is_development {
        return Err(AppError::NotFound("Endpoint unavailable".into()));
    }

    let log = state
        .email_log
        .read()
        .await
        .clone()
        .ok_or_else(|| AppError::NotFound("No verification email found".into()))?;

    Ok(Json(log))
}

pub async fn verify_2fa(
    State(state): State<AppState>,
    Json(body): Json<VerifyRequest>,
) -> Result<Json<Value>, AppError> {
    if body.code.len() != 6 || !body.code.bytes().all(|b| b.is_ascii_digit()) {
        return Err(AppError::BadRequest(
            "Verification code must contain six digits".into(),
        ));
    }

    let mut tx = state.db.begin().await?;

    // Lock the challenge to prevent concurrent reuse.
    let challenge = sqlx::query_as::<_, LoginChallenge>(
        "SELECT * FROM login_challenges
         WHERE id = $1 FOR UPDATE",
    )
    .bind(body.login_challenge_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::BadRequest("Invalid login challenge".into()))?;

    if challenge.used_at.is_some() {
        return Err(AppError::BadRequest(
            "Verification code already used".into(),
        ));
    }

    if challenge.expires_at <= Utc::now() {
        return Err(AppError::BadRequest("Verification code expired".into()));
    }

    if challenge.failed_attempts >= 5 {
        return Err(AppError::TooManyRequests(
            "Too many attempts. Please login again".into(),
        ));
    }

    let valid = verify_password(body.code, challenge.code_hash).await?;

    if !valid {
        sqlx::query(
            "UPDATE login_challenges
             SET failed_attempts = failed_attempts + 1
             WHERE id = $1",
        )
        .bind(challenge.id)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        return Err(AppError::BadRequest("Incorrect verification code".into()));
    }

    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(challenge.user_id)
        .fetch_one(&mut *tx)
        .await?;

    let consumed = sqlx::query(
        "UPDATE login_challenges
         SET used_at = clock_timestamp()
         WHERE id = $1
           AND used_at IS NULL
           AND expires_at > clock_timestamp()",
    )
    .bind(challenge.id)
    .execute(&mut *tx)
    .await?;

    if consumed.rows_affected() != 1 {
        return Err(AppError::BadRequest(
            "Verification code expired or already used".into(),
        ));
    }

    let user_info = UserInfo::from(&user);

    let token = create_token(&user_info, &state.jwt_secret)?;

    tx.commit().await?;

    Ok(Json(json!({
        "access_token": token,
        "user": user_info
    })))
}
