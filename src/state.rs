use crate::db::Db;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Db,
} 