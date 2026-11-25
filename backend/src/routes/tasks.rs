use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    auth::AuthUser,
    error::AppError,
    models::{Task, TaskView},
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,

    #[serde(default)]
    pub description: String,

    pub priority: String,
}

#[derive(Debug, Deserialize)]
pub struct AssignTasksRequest {
    pub task_ids: Vec<Uuid>,
    pub assigned_to_email: String,
}

// POST /tasks
pub async fn create_task(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<CreateTaskRequest>,
) -> Result<(StatusCode, Json<Task>), AppError> {
    auth.require_admin()?;

    let title = body.title.trim();

    if title.is_empty() || title.len() > 200 {
        return Err(AppError::BadRequest(
            "Title must contain 1 to 200 bytes".into(),
        ));
    }

    if !matches!(body.priority.as_str(), "low" | "medium" | "high") {
        return Err(AppError::BadRequest(
            "Priority must be low, medium or high".into(),
        ));
    }

    let task = sqlx::query_as::<_, Task>(
        "INSERT INTO tasks
            (title, description, priority, created_by_id)
         VALUES ($1, $2, $3, $4)
         RETURNING *",
    )
    .bind(title)
    .bind(body.description.trim())
    .bind(&body.priority)
    .bind(auth.0.sub)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(task)))
}

// GET /tasks
pub async fn list_tasks(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<Task>>, AppError> {
    auth.require_admin()?;

    let tasks = sqlx::query_as::<_, Task>(
        "SELECT * FROM tasks
         ORDER BY created_at ASC, id ASC",
    )
    .fetch_all(&state.db)
    .await?;

    Ok(Json(tasks))
}

// POST /tasks/assign
pub async fn assign_tasks(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<AssignTasksRequest>,
) -> Result<Json<Value>, AppError> {
    auth.require_admin()?;

    let mut task_ids = body.task_ids;

    task_ids.sort_unstable();
    task_ids.dedup();

    if task_ids.is_empty() {
        return Err(AppError::BadRequest("Select at least one task".into()));
    }

    let email = body.assigned_to_email.trim().to_lowercase();

    // Coordinate assignment and cache population.
    let mut cache = state.task_cache.write().await;

    let mut tx = state.db.begin().await?;

    let user_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM users
         WHERE email = $1 AND role = 'staff'",
    )
    .bind(&email)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Staff user not found".into()))?;

    // Retrieve previous assignees for cache invalidation.
    let previous_assignees = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT assigned_to_id FROM tasks
             WHERE id = ANY($1)
             FOR UPDATE",
    )
    .bind(&task_ids)
    .fetch_all(&mut *tx)
    .await?;

    if previous_assignees.len() != task_ids.len() {
        return Err(AppError::NotFound("One or more tasks do not exist".into()));
    }

    let result = sqlx::query(
        "UPDATE tasks
         SET assigned_to_id = $1,
             updated_at = NOW()
         WHERE id = ANY($2)",
    )
    .bind(user_id)
    .bind(&task_ids)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    // Invalidate both old and new assignee caches.
    cache.remove(&user_id);

    for previous_id in previous_assignees.into_iter().flatten() {
        cache.remove(&previous_id);
    }

    Ok(Json(json!({
        "message": "Tasks assigned successfully",
        "assigned_to": email,
        "assigned_count": result.rows_affected()
    })))
}

// GET /tasks/view-my-tasks
pub async fn view_my_tasks(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.0.sub;

    // Fast path: read from cache.
    {
        let cache = state.task_cache.read().await;

        if let Some(tasks) = cache.get(&user_id) {
            return Ok(task_response(&auth, tasks, true));
        }
    }

    // Cache miss: acquire write lock and check again.
    let mut cache = state.task_cache.write().await;

    if let Some(tasks) = cache.get(&user_id) {
        return Ok(task_response(&auth, tasks, true));
    }

    let tasks = sqlx::query_as::<_, TaskView>(
        "SELECT
            t.id,
            t.title,
            t.status,
            t.priority,
            u.email AS assigned_to
         FROM tasks t
         JOIN users u ON t.assigned_to_id = u.id
         WHERE t.assigned_to_id = $1
         ORDER BY t.created_at ASC, t.id ASC",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;

    let response = task_response(&auth, &tasks, false);

    cache.insert(user_id, tasks);

    Ok(response)
}

fn task_response(auth: &AuthUser, tasks: &[TaskView], cache_hit: bool) -> Json<Value> {
    Json(json!({
        "user": {
            "email": auth.0.email,
            "role": auth.0.role
        },
        "tasks": tasks,
        "summary": {
            "total_assigned_tasks": tasks.len()
        },
        "cache": {
            "hit": cache_hit
        }
    }))
}
