use super::context::IndexContext;
use crate::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use tera::Context;

pub async fn game_page(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
) -> impl IntoResponse {
    let mut context = Context::new();
    let (game_exists, game_name, player_count) = match state.db.get_game_by_code(&game_code).await {
        Ok(Some(game)) => {
            let players = state
                .db
                .get_players_by_game_id(&*state.db, game.id)
                .await
                .unwrap_or_default();
            (
                true,
                Some(format!("Game {}", game.code)),
                Some(players.len()),
            )
        }
        _ => (false, None, None),
    };
    let index_ctx = IndexContext {
        game_code: Some(game_code.clone()),
        game_exists: Some(game_exists),
        game_name,
        player_count,
        is_game_page: true,
        show_join_modal: game_exists,
        is_rejoin_page: false,
        ..Default::default()
    };
    context.insert("ctx", &index_ctx);
    match state.tera.render("welcome.tera.html", &context) {
        Ok(s) => Html(s).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
