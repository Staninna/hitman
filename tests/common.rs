#![allow(dead_code)]

use hitman::{
    create_router,
    db::Db,
    payloads::{CreateGamePayload, GameCreatedPayload, GameJoinedPayload, JoinGamePayload},
    state::AppState,
};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use uuid::Uuid;

pub async fn spawn_app() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    // Use an in-memory SQLite database for tests
    std::env::set_var("DATABASE_URL", "sqlite::memory:");

    let db = Db::new().await.expect("Failed to create test database");
    let app_state = AppState { db };

    let app = create_router(app_state).layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    );

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    addr
}

#[derive(Debug)]
pub struct TestPlayer {
    pub name: String,
    pub token: String,
    pub secret: Uuid,
}

pub async fn setup_game_with_n_players(
    client: &reqwest::Client,
    addr: &std::net::SocketAddr,
    num_players: usize,
) -> (String, Vec<TestPlayer>) {
    assert!(num_players > 0, "Cannot setup a game with zero players.");

    // 1. Create a game with the first player (host)
    let host_name = format!("Player_{}", 1);
    let create_payload = CreateGamePayload {
        player_name: host_name.clone(),
    };
    let response = client
        .post(&format!("http://{}/game", addr))
        .json(&create_payload)
        .send()
        .await
        .unwrap();
    let game_created: GameCreatedPayload = response.json().await.unwrap();
    let game_code = game_created.game_code;

    let mut players = vec![TestPlayer {
        name: host_name,
        token: game_created.auth_token,
        secret: game_created.player_secret,
    }];

    // 2. Have the rest of the players join
    for i in 2..=num_players {
        let player_name = format!("Player_{}", i);
        let join_payload = JoinGamePayload {
            player_name: player_name.clone(),
        };
        let response = client
            .post(&format!("http://{}/game/{}/join", addr, game_code))
            .json(&join_payload)
            .send()
            .await
            .unwrap();
        let joined: GameJoinedPayload = response.json().await.unwrap();
        players.push(TestPlayer {
            name: player_name,
            token: joined.auth_token,
            secret: joined.player_secret,
        });
    }

    (game_code, players)
}
