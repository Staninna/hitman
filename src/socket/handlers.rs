use crate::{
    errors::AppError,
    payloads::{
        CreateGamePayload, ErrorPayload, GameCreatedPayload, GameJoinedPayload, JoinGamePayload,
        NewTarget, PlayerJoinedPayload, StartGamePayload,
    },
    state::AppState,
};
use socketioxide::extract::{Data, SocketRef};
use tracing::{error, info};

use crate::utils::generate_game_code;

pub async fn create_game(
    socket: SocketRef,
    Data(payload): Data<CreateGamePayload>,
    state: AppState,
) {
    info!("[socket {}] Received create_game: {:?}", socket.id, payload);

    let game_code_len = dotenvy::var("GAME_CODE_LENGTH")
        .unwrap_or_else(|_| "4".to_string())
        .parse::<usize>()
        .expect("GAME_CODE_LENGTH must be a number");

    let game_code = generate_game_code(game_code_len);

    let result = state
        .db
        .create_game(payload.player_name, game_code.clone())
        .await;

    if let Err(e) = result {
        error!("Failed to create game: {}", e);
        send_error(&socket, "Failed to create game.").await;
        return;
    }

    let (game_id, player_id, player_secret, auth_token) = result.unwrap();
    state.connected_players.insert(player_id, socket.clone());

    let players = match state.db.get_players_by_game_id(game_id).await {
        Ok(players) => players,
        Err(e) => {
            error!("Failed to fetch players after game creation: {}", e);
            send_error(&socket, "Game created, but failed to fetch player list.").await;
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

pub async fn join_game(socket: SocketRef, Data(payload): Data<JoinGamePayload>, state: AppState) {
    info!("[socket {}] Received join_game: {:?}", socket.id, payload);

    let result = state
        .db
        .join_game(payload.game_code.clone(), payload.player_name)
        .await;

    let (game_id, player_id, player_secret, auth_token) = match result {
        Ok(Some(data)) => data,
        Ok(None) => {
            error!("Game not found: {}", payload.game_code);
            send_error(&socket, "Game not found.").await;
            return;
        }
        Err(e) => {
            error!("Failed to join game: {}", e);
            send_error(&socket, "Failed to join game.").await;
            return;
        }
    };

    state.connected_players.insert(player_id, socket.clone());

    let players = match state.db.get_players_by_game_id(game_id).await {
        Ok(players) => players,
        Err(e) => {
            error!("Failed to fetch players after joining game: {}", e);
            send_error(&socket, "Joined game, but failed to fetch player list.").await;
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

pub async fn start_game(socket: SocketRef, Data(payload): Data<StartGamePayload>, state: AppState) {
    info!("[socket {}] Received start_game: {:?}", socket.id, payload);

    let game_code = payload.game_code;
    let auth_token = payload.auth_token;

    let player = match state.db.get_player_by_auth_token(&auth_token).await {
        Ok(Some(player)) => player,
        _ => {
            send_error(&socket, "Invalid auth token.").await;
            return;
        }
    };

    // TODO: why we calling start_game twice?
    if let Err(e) = state.db.start_game(&game_code, player.id).await {
        handle_start_game_error(&socket, e).await;
        return;
    }

    let players = state.db.start_game(&game_code, player.id).await.unwrap();

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

async fn handle_start_game_error(socket: &SocketRef, error: AppError) {
    match error {
        AppError::NotFound(message) => {
            error!("Game not found: {}", message);
            send_error(socket, &message).await;
        }
        AppError::Forbidden(message) => {
            error!("Forbidden: {}", message);
            send_error(socket, &message).await;
        }
        _ => {
            error!("Failed to start game");
            send_error(socket, "Failed to start game.").await;
        }
    }
}

async fn send_error(socket: &SocketRef, message: &str) {
    let payload = ErrorPayload {
        message: message.to_string(),
    };
    socket.emit("error", &payload).ok();
}
