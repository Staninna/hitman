use serde::Serialize;

#[derive(Serialize, Default, Clone)]
pub struct IndexContext {
    pub page_name: Option<String>,
    pub game_code: Option<String>,
    pub game_exists: Option<bool>,
    pub game_name: Option<String>,
    pub player_count: Option<usize>,
    pub is_game_page: bool,
    pub show_join_modal: bool,
    pub is_rejoin_page: bool,
    pub auth_token: Option<String>,
    pub player_id: Option<i32>,
    pub player_name: Option<String>,
    pub rejoin_link: Option<String>,
}
