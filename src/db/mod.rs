use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{env, ops::Deref};
use tracing::info;
use dashmap::DashMap;
use std::sync::Arc;

use crate::models::Player;

#[derive(Debug, Clone)]
pub struct Db(pub(crate) SqlitePool, pub(crate) Arc<DashMap<String, crate::models::Player>>);

impl Db {
    /// Initialise a new database connection pool and run migrations.
    pub async fn new() -> Result<Self, sqlx::Error> {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        info!("Connecting to database at {}", db_url);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        info!("Running database migrations...");
        sqlx::migrate!("./migrations").run(&pool).await?;
        info!("Database migrations complete.");

        let player_cache: Arc<DashMap<String, Player>> = Arc::new(DashMap::new());

        Ok(Db(pool, player_cache))
    }
}

// Split implementations into focused modules.
pub mod game;
pub mod player;

// Allow calling methods on the inner pool directly when necessary.
impl Deref for Db {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
