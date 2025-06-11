mod common;

use hitman::{
    models::Player,
    payloads::{KillResponsePayload, StartGamePayload},
};
use reqwest::StatusCode;

async fn setup_started_game(
    client: &reqwest::Client,
    addr: &std::net::SocketAddr,
    num_players: usize,
) -> (String, Vec<common::TestPlayer>) {
    let (game_code, players) = common::setup_game_with_n_players(client, addr, num_players).await;
    let host_token = &players[0].token;

    let start_payload = StartGamePayload {
        auth_token: host_token.clone(),
    };
    client
        .post(&format!("http://{}/game/{}/start", addr, game_code))
        .json(&start_payload)
        .send()
        .await
        .unwrap();

    (game_code, players)
}

async fn get_players(
    client: &reqwest::Client,
    addr: &std::net::SocketAddr,
    game_code: &str,
) -> Vec<Player> {
    client
        .get(&format!("http://{}/game/{}", addr, game_code))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn test_kill_and_target_reassignment() {
    let addr = common::spawn_app().await;
    let client = reqwest::Client::new();
    let (game_code, test_players) = setup_started_game(&client, &addr, 3).await;

    // P1 is the killer
    let killer_test_player = test_players.iter().find(|p| p.name == "Player_1").unwrap();

    // Find P1's target from the game state
    let initial_players = get_players(&client, &addr, &game_code).await;
    let killer_ingame_state = initial_players
        .iter()
        .find(|p| p.name == killer_test_player.name)
        .unwrap();
    let target_name = killer_ingame_state.target_name.as_ref().unwrap();

    // Find the target's secret from our test setup data
    let target_test_player = test_players
        .iter()
        .find(|p| &p.name == target_name)
        .unwrap();

    // P1 kills their target
    let kill_res = client
        .post(&format!(
            "http://{}/game/{}/kill?secret={}",
            addr, game_code, target_test_player.secret
        ))
        .header(
            "Authorization",
            format!("Bearer {}", killer_test_player.token),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(kill_res.status(), StatusCode::OK);

    let kill_response: KillResponsePayload = kill_res.json().await.unwrap();

    assert_eq!(kill_response.killer_name, killer_test_player.name);
    assert_eq!(kill_response.eliminated_player_name, *target_name);
    assert!(!kill_response.game_over);
    assert!(kill_response.new_target_name.is_some());
    assert_ne!(
        kill_response.new_target_name.as_deref(),
        Some(target_name.as_str())
    );

    // Verify the game state reflects the change reported in the response
    let final_players = get_players(&client, &addr, &game_code).await;
    let p1_final = final_players.iter().find(|p| p.name == "Player_1").unwrap();
    assert_eq!(p1_final.target_name, kill_response.new_target_name);
}

#[tokio::test]
async fn test_invalid_kill_wrong_target() {
    let addr = common::spawn_app().await;
    let client = reqwest::Client::new();
    let (game_code, test_players) = setup_started_game(&client, &addr, 3).await;
    let p1_token = &test_players[0].token;

    let initial_players = get_players(&client, &addr, &game_code).await;
    let p1 = initial_players
        .iter()
        .find(|p| p.name == "Player_1")
        .unwrap();
    let p1_target_name = p1.target_name.as_ref().unwrap();

    // Find a player who is NOT P1's target
    let wrong_target_test_player = test_players
        .iter()
        .find(|p| &p.name != p1_target_name && p.name != "Player_1")
        .unwrap();

    let kill_res = client
        .post(&format!(
            "http://{}/game/{}/kill?secret={}",
            addr, game_code, wrong_target_test_player.secret
        ))
        .header("Authorization", format!("Bearer {}", p1_token))
        .send()
        .await
        .unwrap();

    assert_eq!(
        kill_res.status(),
        StatusCode::FORBIDDEN,
        "Should not be able to kill a player who is not the target."
    );
}

#[tokio::test]
async fn test_winning_kill() {
    let addr = common::spawn_app().await;
    let client = reqwest::Client::new();
    let (game_code, test_players) = setup_started_game(&client, &addr, 2).await;

    let winner_test_player = test_players.iter().find(|p| p.name == "Player_1").unwrap();

    // Find winner's target from game state
    let players = get_players(&client, &addr, &game_code).await;
    let winner_initial = players
        .iter()
        .find(|p| p.name == winner_test_player.name)
        .unwrap();
    let loser_name = winner_initial.target_name.as_ref().unwrap();

    // Find loser's secret from test setup data
    let loser_test_player = test_players.iter().find(|p| &p.name == loser_name).unwrap();

    // The winning kill
    let kill_res = client
        .post(&format!(
            "http://{}/game/{}/kill?secret={}",
            addr, game_code, loser_test_player.secret
        ))
        .header(
            "Authorization",
            format!("Bearer {}", winner_test_player.token),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(kill_res.status(), StatusCode::OK);

    let kill_response: KillResponsePayload = kill_res.json().await.unwrap();
    assert_eq!(kill_response.killer_name, winner_test_player.name);
    assert_eq!(kill_response.eliminated_player_name, loser_name.as_str());
    assert!(kill_response.game_over);
    assert!(kill_response.new_target_name.is_none());

    // Verify final state
    let final_players = get_players(&client, &addr, &game_code).await;
    let winner_final = final_players.iter().find(|p| p.name == "Player_1").unwrap();
    let loser_final = final_players
        .iter()
        .find(|p| p.name == loser_name.as_str())
        .unwrap();

    assert!(winner_final.is_alive);
    assert!(!loser_final.is_alive);
    assert!(
        winner_final.target_name.is_none(),
        "Winner should have no target."
    );
}
