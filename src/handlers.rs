use crate::{
    errors::AppError,
    payloads::{GameOver, NewTarget, PlayerEliminated},
    state::AppState,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_extra::headers::{authorization::Bearer, Authorization};
use axum_extra::TypedHeader;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct KillQuery {
    secret: Uuid,
}

pub async fn kill_handler(
    State(state): State<AppState>,
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Query(query): Query<KillQuery>,
) -> Result<impl IntoResponse, AppError> {
    let (killer_name, eliminated_player_name, new_target_name) =
        state.db.process_kill(auth.token(), &query.secret).await?;

    let room = state
        .db
        .get_game_room(&killer_name)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    // Broadcast the elimination to all players in the game
    let eliminated_payload = PlayerEliminated {
        killer_name: killer_name.clone(),
        eliminated_player_name,
    };
    state
        .io
        .to(room.clone())
        .emit("player_eliminated", &eliminated_payload)
        .await
        .ok();

    // Send the new target to the killer
    if let Some(target_name) = new_target_name {
        let new_target_payload = NewTarget { target_name };
        state
            .io
            .to(room)
            .emit("new_target", &new_target_payload)
            .await
            .ok();
    } else {
        // Handle game over logic if there's no new target
        let game_over_payload = GameOver {
            winner_name: killer_name,
        };
        state
            .io
            .to(room)
            .emit("game_over", &game_over_payload)
            .await
            .ok();
    }

    Ok((StatusCode::OK, Json("Kill confirmed")))
}
