use crate::models::Player;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{env, ops::Deref};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Db(SqlitePool);

impl Db {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        info!("Running database migrations...");
        sqlx::migrate!("./migrations").run(&pool).await?;
        info!("Database migrations complete.");

        Ok(Db(pool))
    }

    pub async fn create_game(
        &self,
        player_name: String,
        game_code: String,
    ) -> Result<(i64, Uuid, String), sqlx::Error> {
        let mut tx = self.0.begin().await?;

        let player_secret = Uuid::new_v4();
        let auth_token = Uuid::new_v4().to_string();

        let game_id = sqlx::query!(
            "INSERT INTO games (code, status) VALUES (?, 'lobby') RETURNING id",
            game_code
        )
        .fetch_one(&mut *tx)
        .await?
        .id;

        let player_id = sqlx::query!(
            "INSERT INTO players (game_id, name, secret_code, auth_token) VALUES (?, ?, ?, ?)",
            game_id,
            player_name,
            player_secret,
            auth_token
        )
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();

        sqlx::query!(
            "UPDATE games SET host_id = ? WHERE id = ?",
            player_id,
            game_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((game_id, player_secret, auth_token))
    }

    pub async fn get_players_by_game_id(&self, game_id: i64) -> Result<Vec<Player>, sqlx::Error> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT id, name, secret_code as "secret_code: _", auth_token, is_alive, target_id, game_id
            FROM players
            WHERE game_id = ?
            "#,
            game_id
        )
        .fetch_all(&self.0)
        .await
    }
}

impl Deref for Db {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
} 