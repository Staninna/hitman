use crate::{errors::AppError, state::AppState};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
pub struct ChangedResponse {
    pub changed: bool,
}

pub async fn check_for_changes(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
) -> Result<impl IntoResponse, AppError> {
    let player = state
        .db
        .get_player_by_auth_token(auth.token())
        .await?
        .ok_or(AppError::Forbidden("Invalid auth token.".into()))?;
    let game_map = state.changes.entry(game_code).or_default();
    let mut flag = game_map.entry(player.id).or_insert(true);
    let changed = *flag;
    if changed {
        *flag = false;
    }
    info!("Player {} queried changes -> {}", player.id, changed);
    Ok(Json(ChangedResponse { changed }))
}
