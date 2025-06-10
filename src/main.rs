use axum::{routing::get, Router};
use sqlx::SqlitePool;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::prelude::*;

mod db;
mod models;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "hitman=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_pool = db::create_pool()
        .await
        .expect("Failed to create database pool");

    let app = router(db_pool);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn handler() -> &'static str {
    "Hello, Hitman!"
}

fn router(db_pool: SqlitePool) -> Router {
    Router::new()
        .route("/", get(handler))
        .with_state(db_pool)
}