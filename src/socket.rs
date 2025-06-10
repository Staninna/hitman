use crate::{
    payloads::{CreateGamePayload, ErrorPayload, GameCreatedPayload},
    state::AppState,
};
use rand::prelude::*;
use socketioxide::extract::{AckSender, Data, SocketRef};
use tracing::{error, info};

pub fn on_connect(socket: SocketRef, state: AppState) {
    info!(
        "Socket connected: {} with transport {:?}",
        socket.id,
        socket.transport_type()
    );

    let state_for_create = state.clone();
    socket.on(
        "create_game",
        move |socket: SocketRef, Data::<CreateGamePayload>(payload), _ack: AckSender| {
            let state = state_for_create.clone();
            async move {
                info!("[socket {}] Received create_game: {:?}", socket.id, payload);

                let game_code = generate_game_code(4);

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
                                    message: "Game created, but failed to fetch player list."
                                        .to_string(),
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
        },
    );

    let state_for_join = state.clone();
    socket.on(
        "join_game",
        move |socket: SocketRef, Data::<serde_json::Value>(payload), _ack: AckSender| {
            async move {
                info!(
                    "[socket {}] Received join_game: {:?}, state: {:?}",
                    socket.id,
                    payload,
                    state_for_join.db
                );
                // TODO: Implement game joining logic
            }
        },
    );

    socket.on(
        "start_game",
        move |socket: SocketRef, Data::<serde_json::Value>(payload)| {
            async move {
                info!(
                    "[socket {}] Received start_game: {:?}, state: {:?}",
                    socket.id,
                    payload,
                    state.db
                );
                // TODO: Implement game starting logic
            }
        },
    );
}

fn generate_game_code(len: usize) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut rng = rand::rng();

    CHARSET
        .choose_multiple(&mut rng, len)
        .map(|&c| c as char)
        .collect()
} 