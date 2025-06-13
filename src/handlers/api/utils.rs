use crate::models::Player;
use dashmap::DashMap;

pub fn mark_all_players(
    game_changes: &DashMap<String, DashMap<i64, bool>>,
    game_code: &str,
    players: &[Player],
) {
    let player_marks = game_changes.entry(game_code.to_string()).or_default();
    for p in players {
        player_marks.insert(p.id, true);
    }
}
