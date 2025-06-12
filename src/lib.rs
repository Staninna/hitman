use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

pub mod db;
pub mod errors;
pub mod handlers;
pub mod models;
pub mod payloads;
pub mod state;
pub mod utils;
pub mod frontend_handlers;

use state::AppState;

pub fn create_router(app_state: AppState) -> axum::Router {
    tracing::info!("Creating router");
    Router::new()
        .route("/", get(frontend_handlers::index))
        .route("/events/{game_code}", get(frontend_handlers::sse_handler))
        .nest_service("/static", ServeDir::new("static"))
        .route("/game", post(handlers::create_game))
        .route("/game/{game_code}", get(handlers::get_game_state))
        .route("/game/{game_code}/join", post(handlers::join_game))
        .route("/game/{game_code}/start", post(handlers::start_game))
        .route("/game/{game_code}/kill", post(handlers::kill_handler))
        .with_state(app_state)
}
