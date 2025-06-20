use axum::{
    routing::{get, post},
    Router,
};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

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
    let mut static_path = dotenvy::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    static_path.push_str("/static");

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
        .nest_service("/static", ServeDir::new(static_path))
        .route("/api/game", post(api::create_game))
        .route("/api/game/{game_code}", get(api::get_game_state))
        .route("/api/game/{game_code}/join", post(api::join_game))
        .route("/api/game/{game_code}/start", post(api::start_game))
        .route("/api/game/{game_code}/eliminate", post(api::kill_handler))
        .route("/api/game/{game_code}/leave", post(api::leave_game))
        .with_state(app_state)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &axum::http::Request<_>| {
                let request_id = Uuid::new_v4();
                tracing::info_span!(
                    "request",
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                    request_id = %request_id,
                )
            }),
        )
}
