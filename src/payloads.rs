use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::Player;

// --- Client-to-Server Payloads ---

#[derive(Debug, Deserialize)]
pub struct CreateGamePayload {
    pub player_name: String,
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
pub struct ErrorPayload {
    pub message: String,
} 