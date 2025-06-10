use crate::{
    errors::AppError,
    models::{Game, GameStatus, Player},
};
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

    pub async fn process_kill(
        &self,
        killer_token: &str,
        target_secret: &Uuid,
    ) -> Result<(String, String, Option<String>), AppError> {
        let mut tx = self.0.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {}", e);
            AppError::InternalServerError
        })?;

        // 1. Find killer by auth token
        let killer = sqlx::query_as!(
            Player,
            r#"
            SELECT 
                id as "id!",
                name as "name!",
                secret_code as "secret_code!: Uuid",
                auth_token as "auth_token!",
                is_alive as "is_alive!",
                target_id,
                game_id as "game_id!"
            FROM players
            WHERE auth_token = ?
            "#,
            killer_token
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query for killer: {}", e);
            AppError::InternalServerError
        })?
        .ok_or(AppError::Unauthorized)?;

        if !killer.is_alive {
            return Err(AppError::Forbidden(
                "A dead player cannot perform a kill.".to_string(),
            ));
        }

        // 2. Find target by secret code
        let target = sqlx::query_as!(
            Player,
            r#"
            SELECT
                id as "id!",
                name as "name!",
                secret_code as "secret_code!: Uuid",
                auth_token as "auth_token!",
                is_alive as "is_alive!",
                target_id,
                game_id as "game_id!"
            FROM players
            WHERE secret_code = ?
            "#,
            target_secret
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query for target: {}", e);
            AppError::InternalServerError
        })?
        .ok_or(AppError::NotFound(
            "Target secret does not correspond to an active player.".to_string(),
        ))?;

        if !target.is_alive {
            return Err(AppError::Forbidden(
                "The target is already dead.".to_string(),
            ));
        }

        // 3. Validate game integrity
        if killer.game_id != target.game_id {
            return Err(AppError::Forbidden(
                "Killer and target are not in the same game.".to_string(),
            ));
        }

        if killer.id == target.id {
            return Err(AppError::Forbidden("A player cannot kill themselves.".to_string()));
        }

        // 4. Validate game status
        let game = sqlx::query_as!(
            Game,
            r#"
            SELECT id, code, status as "status: _", host_id, winner_id, created_at
            FROM games
            WHERE id = ?
            "#,
            killer.game_id
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        if game.status != GameStatus::InProgress {
            return Err(AppError::UnprocessableEntity(
                "Game is not in progress.".to_string(),
            ));
        }

        // 5. Validate target
        if killer.target_id != Some(target.id) {
            return Err(AppError::Forbidden(
                "The identified target is not the killer's current target.".to_string(),
            ));
        }

        // 6. Update state
        // Set target's `is_alive` to false
        sqlx::query!(
            "UPDATE players SET is_alive = FALSE WHERE id = ?",
            target.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        // Check for game over condition
        let is_game_over = target.target_id == Some(killer.id) || target.target_id.is_none();
        let new_target_id = if is_game_over { None } else { target.target_id };

        // Update killer's target to the target's old target
        sqlx::query!(
            "UPDATE players SET target_id = ? WHERE id = ?",
            new_target_id,
            killer.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        let new_target_name = if !is_game_over {
            if let Some(new_target_id) = target.target_id {
                sqlx::query_scalar!("SELECT name FROM players WHERE id = ?", new_target_id)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(|_| AppError::InternalServerError)?
            } else {
                None // Should be unreachable due to is_game_over check
            }
        } else {
            // Game is over, set the winner
            sqlx::query!(
                "UPDATE games SET status = 'finished', winner_id = ? WHERE id = ?",
                killer.id,
                killer.game_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|_| AppError::InternalServerError)?;
            None
        };

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            AppError::InternalServerError
        })?;

        Ok((killer.name, target.name, new_target_name))
    }

    pub async fn get_game_room(&self, player_name: &str) -> Result<String, sqlx::Error> {
        let game_code = sqlx::query_scalar!(
            r#"
            SELECT g.code
            FROM games g
            JOIN players p ON g.id = p.game_id
            WHERE p.name = ?
            "#,
            player_name
        )
        .fetch_one(&self.0)
        .await?;
        Ok(game_code)
    }
}

impl Deref for Db {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
