use crate::state::AppState;
use axum::response::Redirect;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
};

pub async fn rejoin_page(
    State(state): State<AppState>,
    Path((game_code, auth_token)): Path<(String, String)>,
) -> impl IntoResponse {
    if let Ok(Some(player)) = state.db.get_player_by_auth_token(&auth_token).await {
        if let Ok(Some(game)) = state.db.get_game_by_id(player.game_id).await {
            if game.code == game_code {
                let url = match game.status.to_string().as_str() {
                    "LOBBY" => format!("/game/{}/player/{}/lobby", game.code, auth_token),
                    "IN_PROGRESS" => {
                        if player.is_alive {
                            format!("/game/{}/player/{}/game", game.code, auth_token)
                        } else {
                            format!("/game/{}/player/{}/eliminated", game.code, auth_token)
                        }
                    }
                    "FINISHED" => format!("/game/{}/player/{}/game_over", game.code, auth_token),
                    _ => "/".into(),
                };
                return Redirect::to(&url).into_response();
            }
        }
    }
    Redirect::to("/").into_response()
}
