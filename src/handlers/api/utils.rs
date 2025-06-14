use crate::state::AppState;

/// Bump the version counter for a game whenever a significant event occurs.
pub fn bump_game_version(state: &AppState, game_code: &str) {
    state.bump_game_version(game_code);
}
