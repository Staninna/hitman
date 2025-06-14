use crate::{errors::AppError, state::AppState};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde::Serialize;
use tracing::info;

#[derive(Serialize)]
pub struct ChangedResponse {
    pub changed: bool,
    pub current_version: i64,
}

#[derive(Deserialize)]
pub struct VersionQuery {
    pub version: Option<i64>,
}

pub async fn check_for_changes(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
    Query(query): Query<VersionQuery>,
) -> Result<impl IntoResponse, AppError> {
    let client_version = query.version.unwrap_or(0);
    let current_version = state.get_game_version(&game_code);
    let changed = current_version > client_version;
    info!(
        "Client version {} vs server {}, changed => {}",
        client_version, current_version, changed
    );
    Ok(Json(ChangedResponse {
        changed,
        current_version,
    }))
}
