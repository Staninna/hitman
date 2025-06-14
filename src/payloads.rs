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

// --- Server-to-Client Payloads ---

#[derive(Debug, Serialize, Deserialize)]
pub struct GameCreatedPayload {
    pub game_code: String,
    pub player_id: i64,
    pub player_secret: Uuid,
    pub auth_token: String,
    pub players: Vec<Player>,
    pub game: Game,
    pub version: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameJoinedPayload {
    pub game_code: String,
    pub player_id: i64,
    pub player_secret: Uuid,
    pub auth_token: String,
    pub players: Vec<Player>,
    pub game: Game,
    pub version: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KillResponsePayload {
    pub eliminated_player_name: String,
    pub killer_name: String,
    pub new_target_name: Option<String>,
    pub game_over: bool,
}
