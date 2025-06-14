use dashmap::DashMap;
use hitman::{create_router, db::Db, state::AppState};
use std::sync::Arc;
use tera::Tera;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    sqlx::any::install_default_drivers();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = Db::new().await.expect("Failed to create database pool");
    let mut template_path = dotenvy::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    template_path.push_str("/templates/**/*");

    let tera = Tera::new(&template_path).expect("Failed to create Tera instance");

    let app_state = AppState {
        db,
        tera,
        versions: Arc::new(DashMap::new()),
    };

    let app = create_router(app_state).layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    );

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
