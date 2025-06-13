use super::utils::mark_all_players;
use crate::{
    errors::AppError,
    payloads::{CreateGamePayload, GameCreatedPayload, GameJoinedPayload, JoinGamePayload},
    state::AppState,
    utils::generate_game_code,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use tracing::info;

pub async fn create_game(
    State(state): State<AppState>,
    Json(payload): Json<CreateGamePayload>,
) -> Result<impl IntoResponse, AppError> {
    info!("Received create_game: {:?}", payload);
    let game_code_len: usize = dotenvy::var("GAME_CODE_LENGTH")
        .unwrap_or_else(|_| "4".into())
        .parse()
        .expect("GAME_CODE_LENGTH must be number");
    let game_code = generate_game_code(game_code_len);
    let (game_id, player_id, player_secret, auth_token) = state
        .db
        .create_game(payload.player_name, game_code.clone())
        .await?;

    let players = state.db.get_players_by_game_id(game_id).await?;
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
    info!("Received join_game {}: {:?}", game_code, payload);
    let (game_id, player_id, player_secret, auth_token) = state
        .db
        .join_game(game_code.clone(), payload.player_name)
        .await?;
    let players = state.db.get_players_by_game_id(game_id).await?;
    let game = state
        .db
        .get_game_by_code(&game_code)
        .await?
        .ok_or(AppError::NotFound("Game not found".into()))?;
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
    info!("Received start_game for {}", game_code);
    let player = state
        .db
        .get_player_by_auth_token(auth.token())
        .await?
        .ok_or(AppError::Forbidden("Invalid auth token.".into()))?;
    let players = state.db.start_game(&game_code, player.id).await?;
    mark_all_players(&state.changes, &game_code, &players);
    Ok(Json(players))
}
