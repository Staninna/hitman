use axum::{routing::post, routing::get, Router};

pub async fn handler() -> &'static str {
    "Hello, Hitman!"
}

pub async fn kill_handler() -> &'static str {
    "Kill confirmed (placeholder)"
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(handler))
        .route("/api/kill", post(kill_handler))
} 