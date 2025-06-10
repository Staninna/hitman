use crate::{
    errors::AppError,
    payloads::{
        CreateGamePayload, ErrorPayload, GameCreatedPayload, GameJoinedPayload, JoinGamePayload,
        NewTarget, PlayerJoinedPayload, StartGamePayload,
    },
    state::AppState,
};
use socketioxide::extract::{AckSender, Data, SocketRef};
use tracing::{error, info};

use crate::utils::generate_game_code;

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
        Ok((game_id, player_id, player_secret, auth_token)) => {
            state.connected_players.insert(player_id, socket.clone());

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
    Data(payload): Data<JoinGamePayload>,
    _ack: AckSender,
    state: AppState,
) {
    info!("[socket {}] Received join_game: {:?}", socket.id, payload);

    match state
        .db
        .join_game(payload.game_code.clone(), payload.player_name)
        .await
    {
        Ok(Some((game_id, player_id, player_secret, auth_token))) => {
            state.connected_players.insert(player_id, socket.clone());

            let players = match state.db.get_players_by_game_id(game_id).await {
                Ok(players) => players,
                Err(e) => {
                    error!("Failed to fetch players after joining game: {}", e);
                    let payload = ErrorPayload {
                        message: "Joined game, but failed to fetch player list.".to_string(),
                    };
                    socket.emit("error", &payload).ok();
                    return;
                }
            };

            let room = payload.game_code.clone();
            let response = GameJoinedPayload {
                game_code: payload.game_code,
                player_secret,
                auth_token,
                players: players.clone(),
            };

            socket.emit("game_joined", &response).ok();

            let player_joined_response = PlayerJoinedPayload { players };
            socket
                .to(room.clone())
                .emit("player_joined", &player_joined_response)
                .await
                .ok();
            socket.join(room);
        }
        Ok(None) => {
            error!("Game not found: {}", payload.game_code);
            let payload = ErrorPayload {
                message: "Game not found.".to_string(),
            };
            socket.emit("error", &payload).ok();
        }
        Err(e) => {
            error!("Failed to join game: {}", e);
            let payload = ErrorPayload {
                message: "Failed to join game.".to_string(),
            };
            socket.emit("error", &payload).ok();
        }
    }
}

pub async fn start_game(socket: SocketRef, Data(payload): Data<StartGamePayload>, state: AppState) {
    info!("[socket {}] Received start_game: {:?}", socket.id, payload);

    let game_code = payload.game_code;
    let auth_token = payload.auth_token;

    let player = match state.db.get_player_by_auth_token(&auth_token).await {
        Ok(Some(player)) => player,
        _ => {
            let payload = ErrorPayload {
                message: "Invalid auth token.".to_string(),
            };
            socket.emit("error", &payload).ok();
            return;
        }
    };

    match state.db.start_game(&game_code, player.id).await {
        Ok(players) => {
            info!("Game {} started with players: {:?}", game_code, players);

            let room = game_code.clone();
            socket.to(room).emit("game_started", &players).await.ok();

            for player in players {
                if let Some(target_name) = player.target_name.clone() {
                    let event = "new_target";
                    let payload = NewTarget { target_name };

                    if let Some(player_socket) = state.connected_players.get(&player.id) {
                        player_socket.emit(event, &payload).ok();
                    }
                }
            }
        }
        Err(AppError::NotFound(message)) => {
            error!("Game not found: {}", message);
            let payload = ErrorPayload { message };
            socket.emit("error", &payload).ok();
        }
        Err(AppError::Forbidden(message)) => {
            error!("Forbidden: {}", message);
            let payload = ErrorPayload { message };
            socket.emit("error", &payload).ok();
        }
        Err(_) => {
            error!("Failed to start game");
            let payload = ErrorPayload {
                message: "Failed to start game.".to_string(),
            };
            socket.emit("error", &payload).ok();
        }
    }
}
