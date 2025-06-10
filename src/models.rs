use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "game_status", rename_all = "lowercase")]
pub enum GameStatus {
    Lobby,
    InProgress,
    Finished,
}

#[derive(Serialize, Deserialize, Debug, Clone, sqlx::FromRow)]
pub struct Player {
    pub id: i64,
    pub name: String,
    #[serde(skip_serializing)] // Never send secrets or tokens to all clients
    pub secret_code: Uuid,
    #[serde(skip_serializing)]
    pub auth_token: String,
    pub is_alive: bool,
    pub target_id: Option<i64>,
    pub game_id: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Game {
    pub id: i64,
    pub code: String,
    pub status: GameStatus,
    pub host_id: Option<i64>,
    pub winner_id: Option<i64>,
    pub created_at: Option<sqlx::types::chrono::NaiveDateTime>,
} 