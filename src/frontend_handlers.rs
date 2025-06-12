use std::convert::Infallible;
use std::time::Duration;
use axum::{
    response::{
        sse::{Event, Sse},
        IntoResponse, Html,
    },
    extract::{State, Path},
};
use serde::Serialize;
use tera::{Context};
use crate::{
    errors::AppError,
    models::{Game, Player},
    state::AppState,
};
use axum::http::StatusCode;

#[derive(Serialize)]
struct GameState {
    game: Game,
    players: Vec<Player>,
}

pub async fn sse_handler(
    Path(game_code): Path<String>,
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            match state.db.get_game_state(&game_code).await {
                Ok(Some((game, players))) => {
                    let game_state = GameState { game, players };
                    let event = Event::default()
                        .data(serde_json::to_string(&game_state).unwrap());
                    yield Ok(event);
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
        }
    };

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}

pub async fn index(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let context = Context::new();
    match state.tera.render("index.html", &context) {
        Ok(s) => Html(s).into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Template error").into_response(),
    }
}