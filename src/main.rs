use axum::routing::post;
use dashmap::DashMap;
use socketioxide::SocketIo;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::prelude::*;

mod db;
mod errors;
mod handlers;
mod models;
mod payloads;
mod socket;
mod state;
mod utils;

use state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    sqlx::any::install_default_drivers();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "hitman=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = db::Db::new().await.expect("Failed to create database pool");

    let (layer, io) = SocketIo::new_layer();

    let state_for_router = AppState {
        db,
        io: io.clone(),
        connected_players: Arc::new(DashMap::new()),
    };
    let state_for_socket = state_for_router.clone();

    io.ns("/", move |socket: socketioxide::extract::SocketRef| {
        socket::on_connect(socket, state_for_socket)
    });

    let app = router(state_for_router).layer(layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn router(app_state: AppState) -> axum::Router {
    axum::Router::new()
        .route("/api/kill", post(handlers::kill_handler))
        .with_state(app_state)
}
