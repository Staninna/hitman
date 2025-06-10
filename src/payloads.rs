use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Player;

// --- Client-to-Server Payloads ---

#[derive(Debug, Deserialize)]
pub struct CreateGamePayload {
    pub player_name: String,
}

#[derive(Debug, Deserialize)]
pub struct JoinGamePayload {
    pub player_name: String,
    pub game_code: String,
}

#[derive(Debug, Deserialize)]
pub struct StartGamePayload {
    pub game_code: String,
    pub auth_token: String,
}

// --- Server-to-Client Payloads ---

#[derive(Debug, Serialize)]
pub struct GameCreatedPayload {
    pub game_code: String,
    pub player_secret: Uuid,
    pub auth_token: String,
    pub players: Vec<Player>,
}

#[derive(Debug, Serialize)]
pub struct GameJoinedPayload {
    pub game_code: String,
    pub player_secret: Uuid,
    pub auth_token: String,
    pub players: Vec<Player>,
}

#[derive(Debug, Serialize)]
pub struct PlayerJoinedPayload {
    pub players: Vec<Player>,
}

#[derive(Debug, Serialize)]
pub struct GameStartedPayload {
    pub players: Vec<Player>,
}

#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub message: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct PlayerEliminated {
    pub eliminated_player_name: String,
    pub killer_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct NewTarget {
    pub target_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct GameOver {
    pub winner_name: String,
}
