use super::context::IndexContext;
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use tera::Context;

pub async fn lobby_page(
    State(state): State<AppState>,
    Path((game_code, auth_token)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut context = Context::new();

    let mut index_context = IndexContext {
        is_rejoin_page: true,
        is_game_page: true,
        game_code: Some(game_code.clone()),
        auth_token: Some(auth_token.clone()),
        ..Default::default()
    };

    if let Ok(Some(player)) = state.db.get_player_by_auth_token(&auth_token).await {
        if let Ok(Some(game)) = state.db.get_game_by_id(player.game_id).await {
            if game.code == game_code {
                index_context.game_exists = Some(true);
                index_context.player_id = Some(player.id);
                index_context.player_name = Some(player.name);
                index_context.rejoin_link =
                    Some(format!("/game/{}/player/{}", game_code, auth_token));
            } else {
                index_context.game_exists = Some(false);
            }
        } else {
            index_context.game_exists = Some(false);
        }
    } else {
        index_context.game_exists = Some(false);
    }

    context.insert("ctx", &index_context);

    match state.tera.render("lobby.html", &context) {
        Ok(s) => Html(s).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}
