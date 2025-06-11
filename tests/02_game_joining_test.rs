mod common;

use reqwest::StatusCode;

#[tokio::test]
async fn test_join_game() {
    let addr = common::spawn_app().await;
    let client = reqwest::Client::new();

    let (game_code, players) = common::setup_game_with_n_players(&client, &addr, 2).await;

    let response = client
        .get(&format!("http://{}/game/{}", addr, game_code))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let players_in_game: Vec<hitman::models::Player> = response.json().await.unwrap();

    assert_eq!(players_in_game.len(), 2);
    assert!(players
        .iter()
        .all(|p| players_in_game.iter().any(|pig| pig.name == p.name)));
}
