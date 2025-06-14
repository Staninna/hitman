use super::utils::bump_game_version;
use crate::{errors::AppError, models::Game, state::AppState};
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
use serde::Serialize;

#[derive(Serialize)]
pub struct PlayerGameState {
    pub id: i64,
    pub name: String,
    pub is_alive: bool,
    pub target_name: Option<String>,
    pub secret_code: Option<String>,
}

#[derive(Serialize)]
pub struct GameStateResponse {
    pub game: Game,
    pub players: Vec<PlayerGameState>,
    pub version: i64,
}

pub async fn get_game_state(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, AppError> {
    let requesting = state
        .db
        .get_player_by_auth_token(auth.token())
        .await?
        .ok_or(AppError::Forbidden("Invalid auth token.".into()))?;
    let (game, players) = state
        .db
        .get_game_state(&game_code)
        .await?
        .ok_or(AppError::NotFound("Game not found".into()))?;
    let players_conv: Vec<PlayerGameState> = players
        .into_iter()
        .map(|p| PlayerGameState {
            id: p.id,
            name: p.name,
            is_alive: p.is_alive,
            target_name: p.target_name,
            secret_code: if p.id == requesting.id {
                Some(p.secret_code)
            } else {
                None
            },
        })
        .collect();
    let version = state.get_game_version(&game_code);
    Ok(Json(GameStateResponse {
        game,
        players: players_conv,
        version,
    }))
}

pub async fn leave_game(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, AppError> {
    state.db.leave_game(&game_code, auth.token()).await?;
    if state.db.get_game_by_code(&game_code).await?.is_some() {
        bump_game_version(&state, &game_code);
    }
    Ok(StatusCode::NO_CONTENT)
}
