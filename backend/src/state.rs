use std::{collections::HashMap, sync::Arc};

use sqlx::PgPool;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{EmailLog, TaskView};

pub type TaskCache = RwLock<HashMap<Uuid, Vec<TaskView>>>;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub jwt_secret: String,
    pub is_development: bool,
    pub task_cache: Arc<TaskCache>,
    pub email_log: Arc<RwLock<Option<EmailLog>>>,
}

impl AppState {
    pub fn new(db: PgPool, jwt_secret: String, is_development: bool) -> Self {
        Self {
            db,
            jwt_secret,
            is_development,
            task_cache: Arc::new(RwLock::new(HashMap::new())),
            email_log: Arc::new(RwLock::new(None)),
        }
    }
}
