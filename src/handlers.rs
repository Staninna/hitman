use crate::{
    errors::AppError, models::Player, payloads::{
        CreateGamePayload, GameCreatedPayload, GameJoinedPayload, JoinGamePayload,
        KillResponsePayload,
    }, state::AppState, utils::generate_game_code
};
use axum::{
    extract::{Path, State},
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

#[derive(Serialize)]
pub struct ChangedResponse {
    changed: bool,
}

pub async fn check_for_changes(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, AppError> {
    let player = match state.db.get_player_by_auth_token(auth.token()).await? {
        Some(player) => player,
        None => return Err(AppError::Forbidden("Invalid auth token.".to_string())),
    };

    let game_map = state.changes.entry(game_code).or_default();
    let mut flag = game_map.entry(player.id).or_insert(true);

    let changed = *flag;
    if changed {
        *flag = false;
    }

    Ok(Json(ChangedResponse { changed }))
}

fn mark_all_players(game_changes: &DashMap<String, DashMap<i64, bool>>, game_code: &str, players: &[Player]) {
    let player_marks = game_changes.entry(game_code.to_string()).or_default();
    for p in players {
        player_marks.insert(p.id, true);
    }
}

#[derive(Deserialize, Debug)]
pub struct KillPayload {
    secret_code: String,
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

    mark_all_players(&state.changes, &game_code, &players);

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

    mark_all_players(&state.changes, &game_code, &players);

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
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received start_game for game {}", game_code);

    let player = match state.db.get_player_by_auth_token(auth.token()).await? {
        Some(player) => {
            debug!("Player found for auth token: {:?}", player);
            player
        }
        None => return Err(AppError::Forbidden("Invalid auth token.".to_string())),
    };

    let players = state.db.start_game(&game_code, player.id).await?;

    info!("Game {} started with players: {:?}", game_code, players);
    debug!("Players after starting game {}: {:?}", game_code, players);

    mark_all_players(&state.changes, &game_code, &players);

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
        if let Some(game) = state.db.get_game_by_code(&game_code).await? {
            let all_players = state.db.get_players_by_game_id(game.id).await?;
            mark_all_players(&state.changes, &game_code, &all_players);
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
pub struct PlayerGameState {
    pub id: i64,
    pub name: String,
    pub is_alive: bool,
    pub target_name: Option<String>,
    pub secret_code: Option<Uuid>,
}

#[derive(Serialize)]
pub struct GameStateResponse {
    pub game: crate::models::Game,
    pub players: Vec<PlayerGameState>,
}

pub async fn get_game_state(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received get_game_state for game {}", game_code);

    let requesting_player = match state.db.get_player_by_auth_token(auth.token()).await? {
        Some(p) => p,
        None => return Err(AppError::Forbidden("Invalid auth token.".to_string())),
    };

    let game = state
        .db
        .get_game_by_code(&game_code)
        .await?
        .ok_or(AppError::NotFound("Game not found".to_string()))?;

    let players_db = state.db.get_players_by_game_id(game.id).await?;
    debug!("Fetched players for game {}: {:?}", game_code, players_db);

    let players: Vec<PlayerGameState> = players_db
        .into_iter()
        .map(|p| PlayerGameState {
            id: p.id,
            name: p.name,
            is_alive: p.is_alive,
            target_name: p.target_name,
            secret_code: if p.id == requesting_player.id {
                Some(p.secret_code)
            } else {
                None
            },
        })
        .collect();

    Ok(Json(GameStateResponse { game, players }))
}

pub async fn leave_game(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, AppError> {
    info!(
        "Received leave_game for game {}: auth_token: {}",
        game_code, auth.token()
    );

    state
        .db
        .leave_game(&game_code, auth.token())
        .await?;

    if let Some(game) = state.db.get_game_by_code(&game_code).await? {
        let all_players = state.db.get_players_by_game_id(game.id).await?;
        mark_all_players(&state.changes, &game_code, &all_players);
    }

    Ok(StatusCode::NO_CONTENT)
}
