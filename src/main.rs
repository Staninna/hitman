use axum::{routing::post, routing::get, Router};
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::prelude::*;

mod db;
mod models;
mod payloads;
mod socket;
mod state;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = db::Db::new()
        .await
        .expect("Failed to create database pool");

    let app_state = AppState { db };

    let (layer, io) = socketioxide::SocketIo::new_layer();

    io.ns(
        "/",
        move |socket: socketioxide::extract::SocketRef| {
            socket::on_connect(socket, app_state.clone())
        },
    );

    let app = router().layer(layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> &'static str {
    "Hello, Hitman!"
}

async fn kill_handler() -> &'static str {
    "Kill confirmed (placeholder)"
}

fn router() -> Router {
    Router::new()
        .route("/", get(handler))
        .route("/api/kill", post(kill_handler))
}