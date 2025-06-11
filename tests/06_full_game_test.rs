mod common;

use hitman::{
    models::Player,
    payloads::{KillResponsePayload, StartGamePayload},
};
use reqwest::StatusCode;

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
async fn test_game_with_many_players() {
    let addr = common::spawn_app().await;
    let client = reqwest::Client::new();
    let num_players = 333;

    let (game_code, test_players) =
        common::setup_game_with_n_players(&client, &addr, num_players).await;
    let host_token = &test_players[0].token;

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

    let num_kills_to_perform = 111;
    let mut num_kills_performed = 0;
    let mut killer_player_index = 0;

    while num_kills_performed < num_kills_to_perform {
        let players_before_kill = get_players(&client, &addr, &game_code).await;
        let killer_test_player = test_players.get(killer_player_index).unwrap();

        let killer_ingame_state = players_before_kill
            .iter()
            .find(|p| p.name == killer_test_player.name)
            .expect("Killer not found in game state");

        if killer_ingame_state.is_alive {
            let target_name = killer_ingame_state.target_name.as_ref().unwrap();

            let target_test_player = test_players
                .iter()
                .find(|p| &p.name == target_name)
                .unwrap();

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

            num_kills_performed += 1;
        }

        killer_player_index += 1;

        if killer_player_index >= test_players.len() {
            panic!("Ran out of players to be killers before reaching the desired number of kills.");
        }
    }

    // Verify the final game state
    let final_players = get_players(&client, &addr, &game_code).await;
    let alive_players = final_players.iter().filter(|p| p.is_alive).count();

    assert_eq!(
        alive_players,
        num_players - num_kills_to_perform,
        "The number of alive players should be the initial number of players minus the number of kills."
    );
} 