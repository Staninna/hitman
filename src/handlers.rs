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
    let (killer_id, killer_name, eliminated_player_name, new_target_name) =
        state.db.process_kill(auth.token(), &query.secret).await?;

    let room = state
        .db
        .get_game_room(&killer_name)
        .await
        .map_err(|_| AppError::InternalServerError)?;

    broadcast_player_eliminated(&state, &room, &killer_name, &eliminated_player_name).await;
    notify_killer_of_new_target_or_game_over(
        &state,
        &room,
        killer_id,
        &killer_name,
        new_target_name,
    )
    .await;

    Ok((StatusCode::OK, Json("Kill confirmed")))
}

async fn broadcast_player_eliminated(
    state: &AppState,
    room: &str,
    killer_name: &str,
    eliminated_player_name: &str,
) {
    let eliminated_payload = PlayerEliminated {
        killer_name: killer_name.to_string(),
        eliminated_player_name: eliminated_player_name.to_string(),
    };
    state
        .io
        .to(room.to_string())
        .emit("player_eliminated", &eliminated_payload)
        .await
        .ok();
}

async fn notify_killer_of_new_target_or_game_over(
    state: &AppState,
    room: &str,
    killer_id: i64,
    killer_name: &str,
    new_target_name: Option<String>,
) {
    if let Some(target_name) = new_target_name {
        notify_new_target(state, killer_id, target_name).await;
    } else {
        notify_game_over(state, room, killer_name).await;
    }
}

async fn notify_new_target(state: &AppState, killer_id: i64, target_name: String) {
    let new_target_payload = NewTarget { target_name };
    if let Some(socket) = state.connected_players.get(&killer_id) {
        socket.emit("new_target", &new_target_payload).ok();
    }
}

async fn notify_game_over(state: &AppState, room: &str, winner_name: &str) {
    let game_over_payload = GameOver {
        winner_name: winner_name.to_string(),
    };
    state
        .io
        .to(room.to_string())
        .emit("game_over", &game_over_payload)
        .await
        .ok();
}
