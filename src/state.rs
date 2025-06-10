use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_pool: SqlitePool,
} 