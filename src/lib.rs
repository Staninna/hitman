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

use handlers::api;
use handlers::frontend as fh;
use state::AppState;

pub fn create_router(app_state: AppState) -> axum::Router {
    tracing::info!("Creating router");
    Router::new()
        // Frontend
        .route("/", get(fh::index))
        .route("/game/{game_code}", get(fh::game_page))
        .route(
            "/game/{game_code}/player/{auth_token}",
            get(fh::rejoin_page),
        )
        .route(
            "/game/{game_code}/player/{auth_token}/lobby",
            get(fh::lobby_page),
        )
        .route(
            "/game/{game_code}/player/{auth_token}/game",
            get(fh::game_in_progress_page),
        )
        .route(
            "/game/{game_code}/player/{auth_token}/eliminated",
            get(fh::eliminated_page),
        )
        .route(
            "/game/{game_code}/player/{auth_token}/game_over",
            get(fh::game_over_page),
        )
        // API
        .route("/api/game/{game_code}/changed", get(api::check_for_changes))
        .nest_service("/static", ServeDir::new("static"))
        .route("/api/game", post(api::create_game))
        .route("/api/game/{game_code}", get(api::get_game_state))
        .route("/api/game/{game_code}/join", post(api::join_game))
        .route("/api/game/{game_code}/start", post(api::start_game))
        .route("/api/game/{game_code}/eliminate", post(api::kill_handler))
        .route("/api/game/{game_code}/leave", post(api::leave_game))
        .with_state(app_state)
}
