mod common;

use hitman::payloads::{CreateGamePayload, GameCreatedPayload};
use reqwest::StatusCode;

#[tokio::test]
async fn test_create_game() {
    let addr = common::spawn_app().await;
    let client = reqwest::Client::new();

    let payload = CreateGamePayload {
        player_name: "The Host".to_string(),
    };

    let response = client
        .post(&format!("http://{}/game", addr))
        .json(&payload)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(response.status(), StatusCode::CREATED);

    let game_created: GameCreatedPayload = response.json().await.unwrap();

    assert!(!game_created.game_code.is_empty());
    assert_eq!(game_created.players.len(), 1);
    assert_eq!(game_created.players[0].name, "The Host");
}
