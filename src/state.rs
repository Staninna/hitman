use crate::db::Db;
use dashmap::DashMap;
use tera::Tera;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub tera: Tera,
    // changes: game_code -> (player_id -> should_refetch)
    pub changes: Arc<DashMap<String, DashMap<i64, bool>>>,
}

impl AppState {
    pub fn debug(&self) {
        // loop through all the games
        for game_ref in self.changes.iter() {
            let game_code = game_ref.key();
            let player_map = game_ref.value();
            // loop through all the players in the game
            for player_ref in player_map.iter() {
                println!("Game code: {}, Player ID: {}, Should refetch: {}", 
                    game_code, player_ref.key(), player_ref.value());
            }
        }
    }
}