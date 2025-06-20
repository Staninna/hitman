use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, sqlx::Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "game_status", rename_all = "lowercase")]
pub enum GameStatus {
    Lobby,
    #[sqlx(rename = "in_progress")]
    InProgress,
    Finished,
}

impl Display for GameStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameStatus::Lobby => write!(f, "LOBBY"),
            GameStatus::InProgress => write!(f, "IN_PROGRESS"),
            GameStatus::Finished => write!(f, "FINISHED"),
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub id: i32,
    pub name: String,
    #[serde(skip)]
    pub secret_code: String,
    #[serde(skip)]
    pub auth_token: String,
    pub is_alive: bool,
    #[serde(skip)]
    pub target_id: Option<i32>,
    #[serde(skip)]
    pub game_id: i32,
    #[serde(default)]
    pub target_name: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow, Serialize, Deserialize)]
pub struct Game {
    pub id: i32,
    pub status: GameStatus,
    pub host_id: Option<i32>,
    pub code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct GameInfo {
    pub code: String,
    pub status: GameStatus,
    pub player_count: i64,
}
