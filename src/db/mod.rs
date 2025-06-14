use sqlx::{postgres::PgPoolOptions, PgPool};
use std::ops::Deref;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Db(PgPool);

impl Db {
    /// Initialise a new database connection pool and run migrations.
    pub async fn new() -> Result<Self, sqlx::Error> {
        let db_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");
        info!("Connecting to database at {}", db_url);

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        info!("Running database migrations...");
        sqlx::migrate!("./migrations").run(&pool).await?;
        info!("Database migrations complete.");

        Ok(Db(pool))
    }
}

// Split implementations into focused modules.
pub mod game;
pub mod player;

// Allow calling methods on the inner pool directly when necessary.
impl Deref for Db {
    type Target = PgPool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
