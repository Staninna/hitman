use crate::db::Db;
use tera::Tera;

#[derive(Clone)]
pub struct AppState {
    pub db: Db,
    pub tera: Tera,
}
