mod common;

use hitman::{models::Player, payloads::StartGamePayload};
use reqwest::StatusCode;

#[tokio::test]
async fn test_start_game_success() {
    let addr = common::spawn_app().await;
    let client = reqwest::Client::new();
    let (game_code, players) = common::setup_game_with_n_players(&client, &addr, 2).await;
    let host_token = &players.iter().find(|p| p.name == "Player_1").unwrap().token;

    let start_payload = StartGamePayload {
        auth_token: host_token.to_string(),
    };
    let response = client
        .post(&format!("http://{}/game/{}/start", addr, game_code))
        .json(&start_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let players: Vec<Player> = response.json().await.unwrap();
    let host = players.iter().find(|p| p.name == "Player_1").unwrap();
    let player_2 = players.iter().find(|p| p.name == "Player_2").unwrap();

    assert!(host.target_name.is_some(), "Host should have a target.");
    assert!(
        player_2.target_name.is_some(),
        "Player 2 should have a target."
    );
    assert_ne!(
        host.target_name, player_2.target_name,
        "Targets should be different."
    );
}

#[tokio::test]
async fn test_start_game_by_non_host_fails() {
    let addr = common::spawn_app().await;
    let client = reqwest::Client::new();
    let (game_code, players) = common::setup_game_with_n_players(&client, &addr, 2).await;
    let non_host_token = &players.iter().find(|p| p.name == "Player_2").unwrap().token;

    let start_payload = StartGamePayload {
        auth_token: non_host_token.to_string(),
    };
    let response = client
        .post(&format!("http://{}/game/{}/start", addr, game_code))
        .json(&start_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.status(),
        StatusCode::FORBIDDEN,
        "Non-host player should not be able to start the game."
    );
}
