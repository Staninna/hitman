use crate::{
    payloads::{CreateGamePayload, ErrorPayload, GameCreatedPayload},
    state::AppState,
};
use socketioxide::extract::{AckSender, Data, SocketRef};
use tracing::{error, info};

use super::utils::generate_game_code;

pub async fn create_game(
    socket: SocketRef,
    Data(payload): Data<CreateGamePayload>,
    _ack: AckSender,
    state: AppState,
) {
    info!("[socket {}] Received create_game: {:?}", socket.id, payload);

    let game_code_len = dotenvy::var("GAME_CODE_LENGTH")
        .unwrap_or_else(|_| "4".to_string())
        .parse::<usize>()
        .expect("GAME_CODE_LENGTH must be a number");

    let game_code = generate_game_code(game_code_len);

    match state
        .db
        .create_game(payload.player_name, game_code.clone())
        .await
    {
        Ok((game_id, player_secret, auth_token)) => {
            let players = match state.db.get_players_by_game_id(game_id).await {
                Ok(players) => players,
                Err(e) => {
                    error!("Failed to fetch players after game creation: {}", e);
                    let payload = ErrorPayload {
                        message: "Game created, but failed to fetch player list.".to_string(),
                    };
                    socket.emit("error", &payload).ok();
                    return;
                }
            };

            let response = GameCreatedPayload {
                game_code,
                player_secret,
                auth_token,
                players,
            };

            socket.emit("game_created", &response).ok();
        }
        Err(e) => {
            error!("Failed to create game: {}", e);
            let payload = ErrorPayload {
                message: "Failed to create game.".to_string(),
            };
            socket.emit("error", &payload).ok();
        }
    }
}

pub async fn join_game(
    socket: SocketRef,
    Data(payload): Data<serde_json::Value>,
    _ack: AckSender,
    state: AppState,
) {
    info!(
        "[socket {}] Received join_game: {:?}, state: {:?}",
        socket.id, payload, state.db
    );
    // TODO: Implement game joining logic
}

pub async fn start_game(
    socket: SocketRef,
    Data(payload): Data<serde_json::Value>,
    state: AppState,
) {
    info!(
        "[socket {}] Received start_game: {:?}, state: {:?}",
        socket.id, payload, state.db
    );
    // TODO: Implement game starting logic
}
