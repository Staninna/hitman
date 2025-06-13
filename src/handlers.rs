use crate::{
    errors::AppError,
    payloads::{
        CreateGamePayload, GameCreatedPayload, GameJoinedPayload, JoinGamePayload,
        KillResponsePayload, StartGamePayload,
    },
    state::AppState,
    utils::generate_game_code,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use serde::{Deserialize, Serialize};
use dashmap::DashMap;
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ChangedParams {
    player_id: i64,
}

#[derive(Serialize)]
pub struct ChangedResponse {
    changed: bool,
}

pub async fn check_for_changes(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    Query(params): Query<ChangedParams>,
) -> Result<impl IntoResponse, AppError> {
    let player_id = params.player_id;

    let mut changed = true;

    if let Some(game_map) = state.changes.get(&game_code) {
        if let Some(mut flag) = game_map.get_mut(&player_id) {
            if *flag {
                *flag = false;
                changed = true;
            } else {
                changed = false;
            }
        } else {
            game_map.insert(player_id, false);
        }
    } else {
        let player_map = DashMap::new();
        player_map.insert(player_id, false);
        state.changes.insert(game_code, player_map);
    }

    Ok(Json(ChangedResponse { changed }))
}

#[derive(Deserialize, Debug)]
pub struct KillPayload {
    secret_code: String,
}

fn mark_all_players(game_changes: &DashMap<i64, bool>, players: &[crate::models::Player]) {
    for p in players {
        game_changes.insert(p.id, true);
    }
}

pub async fn create_game(
    State(state): State<AppState>,
    Json(payload): Json<CreateGamePayload>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received create_game: {:?}", payload);

    let game_code_len = dotenvy::var("GAME_CODE_LENGTH")
        .unwrap_or_else(|_| "4".to_string())
        .parse::<usize>()
        .expect("GAME_CODE_LENGTH must be a number");

    let game_code = generate_game_code(game_code_len);
    debug!("Generated game code: {}", game_code);

    let (game_id, player_id, player_secret, auth_token) = state
        .db
        .create_game(payload.player_name, game_code.clone())
        .await?;

    debug!(
        "Game created with id: {}, player_secret: {}, auth_token: {}",
        game_id, player_secret, auth_token
    );

    let players = state.db.get_players_by_game_id(game_id).await?;
    debug!("Fetched players: {:?}", players);

    let game = state
        .db
        .get_game_by_code(&game_code)
        .await?
        .ok_or(AppError::InternalServerError)?;

    {
        let game_changes = state
            .changes
            .entry(game_code.clone())
            .or_insert_with(DashMap::new);
        mark_all_players(&game_changes, &players);
    }

    let response = GameCreatedPayload {
        game_code,
        player_id,
        player_secret,
        auth_token,
        players,
        game,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn join_game(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    Json(payload): Json<JoinGamePayload>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received join_game for game {}: {:?}", game_code, payload);

    let (game_id, player_id, player_secret, auth_token) = state
        .db
        .join_game(game_code.clone(), payload.player_name)
        .await?;

    debug!(
        "Player joined game with id: {}, player_secret: {}, auth_token: {}",
        game_id, player_secret, auth_token
    );

    let players = state.db.get_players_by_game_id(game_id).await?;
    debug!("Fetched players for game {}: {:?}", game_code, players);

    let game = state
        .db
        .get_game_by_code(&game_code)
        .await?
        .ok_or(AppError::NotFound("Game not found".to_string()))?;

    {
        let game_changes = state
            .changes
            .entry(game_code.clone())
            .or_insert_with(DashMap::new);
        mark_all_players(&game_changes, &players);
    }

    let response = GameJoinedPayload {
        game_code,
        player_id,
        player_secret,
        auth_token,
        players,
        game,
    };

    Ok(Json(response))
}

pub async fn start_game(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    Json(payload): Json<StartGamePayload>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received start_game for game {}: {:?}", game_code, payload);

    let auth_token = payload.auth_token;

    let player = match state.db.get_player_by_auth_token(&auth_token).await? {
        Some(player) => {
            debug!("Player found for auth token: {:?}", player);
            player
        }
        None => return Err(AppError::Forbidden("Invalid auth token.".to_string())),
    };

    let players = state.db.start_game(&game_code, player.id).await?;

    info!("Game {} started with players: {:?}", game_code, players);
    debug!("Players after starting game {}: {:?}", game_code, players);

    {
        let game_changes = state
            .changes
            .entry(game_code.clone())
            .or_insert_with(DashMap::new);
        mark_all_players(&game_changes, &players);
    }

    Ok(Json(players))
}

pub async fn kill_handler(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<KillPayload>,
) -> Result<impl IntoResponse, AppError> {
    info!(
        "Received kill_handler for game {}: killer token: {}, secret: {}",
        game_code,
        auth.token(),
        payload.secret_code
    );

    let secret = Uuid::parse_str(&payload.secret_code)
        .map_err(|_| AppError::UnprocessableEntity("Invalid secret code format".to_string()))?;

    let (killer_id, killer_name, eliminated_player_name, new_target_name) = state
        .db
        .process_kill(&game_code, auth.token(), &secret)
        .await?;

    {
        let game_changes = state
            .changes
            .entry(game_code.clone())
            .or_insert_with(DashMap::new);
        let game_opt = state.db.get_game_by_code(&game_code).await?;
        if let Some(game) = game_opt {
            let all_players = state.db.get_players_by_game_id(game.id).await?;
            mark_all_players(&game_changes, &all_players);
        }
    }

    debug!(
        "Processed kill in game {}: killer_id: {}, killer_name: {}, eliminated_player_name: {}, new_target_name: {:?}",
        game_code, killer_id, killer_name, eliminated_player_name, new_target_name
    );

    let response = KillResponsePayload {
        eliminated_player_name,
        killer_name,
        game_over: new_target_name.is_none(),
        new_target_name,
    };

    Ok((StatusCode::OK, Json(response)))
}

#[derive(Serialize)]
pub struct GameStateResponse {
    game: crate::models::Game,
    players: Vec<crate::models::Player>,
}

pub async fn get_game_state(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received get_game_state for game {}", game_code);
    let game = state
        .db
        .get_game_by_code(&game_code)
        .await?
        .ok_or(AppError::NotFound("Game not found".to_string()))?;

    let players = state.db.get_players_by_game_id(game.id).await?;
    debug!("Fetched players for game {}: {:?}", game_code, players);
    Ok(Json(GameStateResponse { game, players }))
}

pub async fn leave_game(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    Json(payload): Json<crate::payloads::LeaveGamePayload>,
) -> Result<impl IntoResponse, AppError> {
    info!(
        "Received leave_game for game {}: auth_token: {}",
        game_code, payload.auth_token
    );

    state.db.leave_game(&game_code, &payload.auth_token).await?;

    {
        let game_changes = state
            .changes
            .entry(game_code.clone())
            .or_insert_with(DashMap::new);
        let game_opt = state.db.get_game_by_code(&game_code).await?;
        if let Some(game) = game_opt {
            let all_players = state.db.get_players_by_game_id(game.id).await?;
            mark_all_players(&game_changes, &all_players);
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
