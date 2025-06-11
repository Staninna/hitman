mod common;

use hitman::{models::Player, payloads::StartGamePayload};
use reqwest::StatusCode;
use std::collections::HashSet;

#[tokio::test]
async fn test_large_game_target_assignment() {
    let addr = common::spawn_app().await;
    let client = reqwest::Client::new();
    let num_players = 30;

    let (game_code, players) = common::setup_game_with_n_players(&client, &addr, num_players).await;
    let host_token = &players[0].token;

    // The host starts the game
    let start_payload = StartGamePayload {
        auth_token: host_token.clone(),
    };
    let response = client
        .post(&format!("http://{}/game/{}/start", addr, game_code))
        .json(&start_payload)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Get the game state to verify targets
    let response = client
        .get(&format!("http://{}/game/{}", addr, game_code))
        .send()
        .await
        .unwrap();

    let players: Vec<Player> = response.json().await.unwrap();
    assert_eq!(players.len(), num_players);

    // --- Verification ---

    // a. Verify every player has a target
    assert!(
        players.iter().all(|p| p.target_name.is_some()),
        "Not all players were assigned a target."
    );

    // b. Verify all targets are unique (no one is targeted by more than one person)
    let mut target_names = HashSet::new();
    for player in &players {
        let target_name = player.target_name.as_ref().unwrap();
        assert!(
            target_names.insert(target_name),
            "Duplicate target found: {}",
            target_name
        );
    }
    assert_eq!(
        target_names.len(),
        num_players,
        "The number of unique targets should match the number of players."
    );

    // c. Verify no two players target each other
    for player_a in &players {
        let target_of_a = player_a.target_name.as_ref().unwrap();

        // Find player B, who is player A's target
        let player_b = players
            .iter()
            .find(|p| &p.name == target_of_a)
            .expect("Target player not found in the list of players.");

        let target_of_b = player_b.target_name.as_ref().unwrap();

        assert_ne!(
            &player_a.name, target_of_b,
            "Found a reciprocal target: {} and {} are targeting each other.",
            player_a.name, player_b.name
        );
    }
}
