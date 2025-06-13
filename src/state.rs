use crate::db::Db;
use dashmap::DashMap;
use std::time::SystemTime;
use tera::Tera;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub tera: Tera,
    pub changes: DashMap<String, SystemTime>,
}
