pub mod context;
pub mod eliminated;
pub mod game_over;
pub mod game_page;
pub mod in_progress;
pub mod index;
pub mod lobby;
pub mod rejoin;
// Additional page modules would follow similarly (lobby, in_progress, eliminated, game_over)

pub use context::IndexContext;
pub use eliminated::eliminated_page;
pub use game_over::game_over_page;
pub use game_page::game_page;
pub use in_progress::game_in_progress_page;
pub use index::index;
pub use lobby::lobby_page;
pub use rejoin::rejoin_page;
