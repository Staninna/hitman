use crate::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use serde::Serialize;
use tera::Context;

#[derive(Serialize)]
#[derive(Default)]
struct IndexContext {
    game_code: Option<String>,
    game_exists: Option<bool>,
    game_name: Option<String>,
    player_count: Option<usize>,
    is_game_page: bool,
    show_join_modal: bool,
    is_rejoin_page: bool,
    auth_token: Option<String>,
    player_id: Option<i64>,
    player_name: Option<String>,
    rejoin_link: Option<String>,
}


pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let mut context = Context::new();

    let index_context = IndexContext {
        is_game_page: false,
        show_join_modal: false,
        is_rejoin_page: false,
        ..Default::default()
    };

    context.insert("ctx", &index_context);

    match state.tera.render("welcome.html", &context) {
        Ok(s) => Html(s).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn game_page(
    State(state): State<AppState>,
    Path(game_code): Path<String>,
) -> impl IntoResponse {
    let mut context = Context::new();

    // Check if the game exists and get its details
    let (game_exists, game_name, player_count) = match state.db.get_game_by_code(&game_code).await {
        Ok(Some(game)) => {
            let players = state
                .db
                .get_players_by_game_id(game.id)
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

    let index_context = IndexContext {
        game_code: Some(game_code.clone()),
        game_exists: Some(game_exists),
        game_name,
        player_count,
        is_game_page: true,
        show_join_modal: game_exists, // Only show join modal if game exists
        is_rejoin_page: false,
        ..Default::default()
    };

    context.insert("ctx", &index_context);

    match state.tera.render("join_game.html", &context) {
        Ok(s) => Html(s).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

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
                    _ => "/".to_string(),
                };
                return axum::response::Redirect::to(&url).into_response();
            }
        }
    }
    axum::response::Redirect::to("/").into_response()
}

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

pub async fn game_in_progress_page(
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

    match state.tera.render("game.html", &context) {
        Ok(s) => Html(s).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn eliminated_page(
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

    match state.tera.render("eliminated.html", &context) {
        Ok(s) => Html(s).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn game_over_page(
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

    match state.tera.render("game_over.html", &context) {
        Ok(s) => Html(s).into_response(),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}
