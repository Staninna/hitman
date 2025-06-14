use super::utils::bump_game_version;
use crate::{errors::AppError, payloads::KillResponsePayload, state::AppState};
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
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize, Debug)]
pub struct KillPayload {
    secret_code: String,
}

pub async fn kill_handler(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(payload): Json<KillPayload>,
) -> Result<impl IntoResponse, AppError> {
    info!("kill_handler {}", game_code);
    let (_killer_id, killer_name, eliminated, new_target) = state
        .db
        .process_kill(&game_code, auth.token(), &payload.secret_code)
        .await?;
    if state.db.get_game_by_code(&game_code).await?.is_some() {
        bump_game_version(&state, &game_code);
    }
    let resp = KillResponsePayload {
        eliminated_player_name: eliminated,
        killer_name,
        game_over: new_target.is_none(),
        new_target_name: new_target,
    };
    Ok((StatusCode::OK, Json(resp)))
}
