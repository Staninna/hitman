use crate::db::Db;
use dashmap::DashMap;
use socketioxide::{extract::SocketRef, SocketIo};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Db,
    pub io: SocketIo,
    pub connected_players: Arc<DashMap<i64, SocketRef>>,
}
