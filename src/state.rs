use crate::db::Db;
use socketioxide::SocketIo;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db: Db,
    pub io: SocketIo,
}
