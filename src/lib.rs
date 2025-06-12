use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;

pub mod db;
pub mod errors;
pub mod frontend_handlers;
pub mod handlers;
pub mod models;
pub mod payloads;
pub mod state;
pub mod utils;

use state::AppState;

pub fn create_router(app_state: AppState) -> axum::Router {
    tracing::info!("Creating router");
    Router::new()
        .route("/", get(frontend_handlers::index))
        .route("/game/{game_code}", get(frontend_handlers::game_page))
        .route(
            "/game/{game_code}/player/{auth_token}",
            get(frontend_handlers::rejoin_page),
        )
        .route("/events/{game_code}", get(frontend_handlers::sse_handler))
        .nest_service("/static", ServeDir::new("static"))
        .route("/api/game", post(handlers::create_game))
        .route("/api/game/{game_code}", get(handlers::get_game_state))
        .route("/api/game/{game_code}/join", post(handlers::join_game))
        .route("/api/game/{game_code}/start", post(handlers::start_game))
        .route(
            "/api/game/{game_code}/eliminate",
            post(handlers::kill_handler),
        )
        .route("/api/game/{game_code}/leave", post(handlers::leave_game))
        .with_state(app_state)
}
