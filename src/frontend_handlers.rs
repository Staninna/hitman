use crate::{models::Game, state::AppState};
use axum::http::StatusCode;
use axum::{
    extract::{Path, Query, State},
    response::{
        sse::{Event, Sse},
        Html, IntoResponse,
    },
};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tera::Context;
use uuid::Uuid;

#[derive(Serialize)]
struct GameState {
    game: Game,
    players: Vec<PlayerSse>,
}

#[derive(Serialize, Clone)]
struct PlayerSse {
    id: i64,
    name: String,
    is_alive: bool,
    target_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    secret_code: Option<Uuid>,
}

#[derive(Deserialize)]
pub struct SseParams {
    player_id: Option<i64>,
    auth_token: Option<String>,
}

#[derive(Serialize)]
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

impl Default for IndexContext {
    fn default() -> Self {
        Self {
            game_code: None,
            game_exists: None,
            game_name: None,
            player_count: None,
            is_game_page: false,
            show_join_modal: false,
            is_rejoin_page: false,
            auth_token: None,
            player_id: None,
            player_name: None,
            rejoin_link: None,
        }
    }
}

// TODO: Only send events on connection and when the game state changes
// TODO: auth token is needed otherwise the player can see the secret code of other players by changing player_id
pub async fn sse_handler(
    Path(game_code): Path<String>,
    State(state): State<AppState>,
    Query(params): Query<SseParams>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let mut last_state = String::new();

    let stream = async_stream::stream! {
        loop {
            match state.db.get_game_state(&game_code).await {
                Ok(Some((game, players))) => {
                    let mut sse_players: Vec<PlayerSse> = players.iter().map(|p| PlayerSse {
                        id: p.id,
                        name: p.name.clone(),
                        is_alive: p.is_alive,
                        target_name: None,
                        secret_code: None,
                    }).collect();

                    if let (Some(player_id), Some(auth_token)) = (params.player_id, params.auth_token.as_ref()) {
                        if let Some(me_full) = players.iter().find(|p| p.id == player_id) {
                            if me_full.auth_token == *auth_token {
                                if let Some(me_sse) = sse_players.iter_mut().find(|p| p.id == player_id) {
                                    me_sse.target_name = me_full.target_name.clone();
                                    me_sse.secret_code = Some(me_full.secret_code);
                                }
                            }
                        }
                    }

                    let game_state = GameState { game, players: sse_players };
                    if let Ok(current_state) = serde_json::to_string(&game_state) {
                        if last_state != current_state {
                            let event = Event::default().data(current_state.clone());
                            yield Ok(event);
                            last_state = current_state;
                        }
                    }
                }
                Ok(None) => {
                    let event = Event::default().event("error").data("Game not found");
                    yield Ok(event);
                    break;
                }
                Err(_) => {
                    let event = Event::default().event("error").data("Database error");
                    yield Ok(event);
                    break;
                }
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
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
            // Check if the player belongs to the game in the URL
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
