use crate::db::Db;
use dashmap::DashMap;
use std::sync::Arc;
use tera::Tera;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub tera: Tera,
    pub versions: Arc<DashMap<String, i64>>,
}

impl AppState {
    pub fn bump_game_version(&self, game_code: &str) -> i64 {
        let mut entry = self.versions.entry(game_code.to_string()).or_insert(0);
        *entry.value_mut() += 1;
        *entry
    }

    /// Fetch the current version for a game (defaults to 0 if never seen).
    pub fn get_game_version(&self, game_code: &str) -> i64 {
        self.versions.get(game_code).map(|v| *v.value()).unwrap_or(0)
    }
}
