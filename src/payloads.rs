use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Game, Player};

// --- Client-to-Server Payloads ---

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateGamePayload {
    pub player_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JoinGamePayload {
    pub player_name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StartGamePayload {
    pub auth_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LeaveGamePayload {
    pub auth_token: String,
}

// --- Server-to-Client Payloads ---

#[derive(Debug, Serialize, Deserialize)]
pub struct GameCreatedPayload {
    pub game_code: String,
    pub player_id: i64,
    pub player_secret: Uuid,
    pub auth_token: String,
    pub players: Vec<Player>,
    pub game: Game,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameJoinedPayload {
    pub game_code: String,
    pub player_id: i64,
    pub player_secret: Uuid,
    pub auth_token: String,
    pub players: Vec<Player>,
    pub game: Game,
}

#[derive(Debug, Serialize)]
pub struct PlayerJoinedPayload {
    pub players: Vec<Player>,
}

#[derive(Debug, Serialize)]
pub struct PlayerLeftPayload {
    pub player_name: String,
}

#[derive(Debug, Serialize)]
pub struct PlayerReconnectedPayload {
    pub player_name: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct KillResponsePayload {
    pub eliminated_player_name: String,
    pub killer_name: String,
    pub new_target_name: Option<String>,
    pub game_over: bool,
}
