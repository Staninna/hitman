use crate::{
    errors::AppError,
    models::{Game, GameStatus, Player},
};
use rand::seq::SliceRandom;
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
    ) -> Result<(i64, i64, Uuid, String), sqlx::Error> {
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

        Ok((game_id, player_id, player_secret, auth_token))
    }

    pub async fn join_game(
        &self,
        game_code: String,
        player_name: String,
    ) -> Result<(i64, i64, Uuid, String), AppError> {
        let mut tx = self.0.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {}", e);
            AppError::InternalServerError
        })?;

        let game = sqlx::query_as!(
            Game,
            r#"SELECT id as "id!", code, status as "status: _", host_id, winner_id, created_at FROM games WHERE code = ?"#,
            game_code
        )
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| AppError::InternalServerError)?
            .ok_or(AppError::NotFound("Game not found".to_string()))?;

        if game.status != GameStatus::Lobby {
            return Err(AppError::UnprocessableEntity(
                "You can not join games that aren't in lobby".to_string(),
            ));
        }

        if let Some(player) = self
            .get_player_by_name(game.id, &player_name)
            .await
            .map_err(|_| AppError::InternalServerError)?
        {
            if player.is_alive {
                // This is a reconnection
                tx.commit()
                    .await
                    .map_err(|_| AppError::InternalServerError)?;
                return Ok((
                    player.game_id,
                    player.id,
                    player.secret_code,
                    player.auth_token,
                ));
            } else {
                // Player is in the game but is dead
                return Err(AppError::Forbidden(
                    "You have been eliminated from this game.".to_string(),
                ));
            }
        }

        // New player
        let player_secret = Uuid::new_v4();
        let auth_token = Uuid::new_v4().to_string();

        let player_id = sqlx::query!(
            "INSERT INTO players (game_id, name, secret_code, auth_token) VALUES (?, ?, ?, ?)",
            game.id,
            player_name,
            player_secret,
            auth_token
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::UnprocessableEntity("Player name already taken".to_string());
                }
            }
            tracing::error!("Failed to insert player: {}", e);
            AppError::InternalServerError
        })?
        .last_insert_rowid();

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            AppError::InternalServerError
        })?;

        Ok((game.id, player_id, player_secret, auth_token))
    }

    pub async fn get_players_by_game_id(&self, game_id: i64) -> Result<Vec<Player>, sqlx::Error> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT id as "id!", name, secret_code as "secret_code: _", auth_token, is_alive, target_id, game_id, NULL as "target_name: _"
            FROM players
            WHERE game_id = ?
            "#,
            game_id
        )
        .fetch_all(&self.0)
        .await
    }

    pub async fn get_player_by_auth_token(
        &self,
        auth_token: &str,
    ) -> Result<Option<Player>, sqlx::Error> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT id as "id!", name, secret_code as "secret_code: _", auth_token, is_alive, target_id, game_id, NULL as "target_name: _"
            FROM players
            WHERE auth_token = ?
            "#,
            auth_token
        )
        .fetch_optional(&self.0)
        .await
    }

    pub async fn start_game(
        &self,
        game_code: &str,
        player_id: i64,
    ) -> Result<Vec<Player>, AppError> {
        let mut tx = self
            .0
            .begin()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let game = sqlx::query_as!(
            Game,
            r#"SELECT id as "id!", code, status as "status: _", host_id, winner_id, created_at FROM games WHERE code = ?"#,
            game_code
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or_else(|| AppError::NotFound("Game not found.".to_string()))?;

        if game.host_id != Some(player_id) {
            return Err(AppError::Forbidden(
                "Only the host can start the game.".to_string(),
            ));
        }

        if game.status != GameStatus::Lobby {
            return Err(AppError::UnprocessableEntity(
                "Game has already started or has finished.".to_string(),
            ));
        }

        let mut players = self
            .get_players_by_game_id(game.id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if players.len() < 2 {
            return Err(AppError::UnprocessableEntity(
                "Not enough players to start the game.".to_string(),
            ));
        }

        players.shuffle(&mut rand::rng());

        // This assigns each player a target in a circular fashion.
        // For example, with 3 players (A, B, C), A targets B, B targets C, and C targets A.
        // This ensures that no player targets themselves and each player has exactly one target.
        // In the case of 2 players, they will target each other.
        for i in 0..players.len() {
            let target_index = (i + 1) % players.len();
            players[i].target_id = Some(players[target_index].id);
        }

        for player in &players {
            if let Some(target_id) = player.target_id {
                sqlx::query!(
                    "UPDATE players SET target_id = ? WHERE id = ?",
                    target_id,
                    player.id
                )
                .execute(&mut *tx)
                .await
                .map_err(|_| AppError::InternalServerError)?;
            } else {
                // This branch should be unreachable, as target assignment ensures every player
                // has a target. If this code is ever executed, it indicates a bug in the
                // target assignment logic.
                unreachable!("All players must have a target before starting the game.");
            }
        }

        sqlx::query!(
            "UPDATE games SET status = 'in_progress' WHERE id = ?",
            game.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        tx.commit()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let started_players = self
            .get_players_by_game_id(game.id)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let mut players_with_targets = Vec::new();
        for mut player in players {
            let target = started_players
                .iter()
                .find(|p| p.id == player.target_id.unwrap())
                .unwrap();
            player.target_name = Some(target.name.clone());
            players_with_targets.push(player);
        }

        Ok(players_with_targets)
    }

    pub async fn process_kill(
        &self,
        killer_token: &str,
        target_secret: &Uuid,
    ) -> Result<(i64, String, String, Option<String>), AppError> {
        let mut tx = self.0.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {}", e);
            AppError::InternalServerError
        })?;

        // 1. Find killer by auth token
        let killer = self
            .get_player_by_auth_token_in_tx(&mut tx, killer_token)
            .await?;

        // 2. Find target by secret code
        let target = self
            .get_player_by_secret_in_tx(&mut tx, target_secret)
            .await?;

        // 3. Validate game and kill
        let game = self.get_game_by_id_in_tx(&mut tx, killer.game_id).await?;
        Self::validate_kill(&killer, &target, &game)?;

        // 4. Update state
        let new_target_name = self
            .update_game_state_after_kill(&mut tx, &killer, &target)
            .await?;

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            AppError::InternalServerError
        })?;

        Ok((killer.id, killer.name, target.name, new_target_name))
    }

    async fn get_player_by_auth_token_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        killer_token: &str,
    ) -> Result<Player, AppError> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT 
                id as "id!",
                name as "name!",
                secret_code as "secret_code!: Uuid",
                auth_token as "auth_token!",
                is_alive as "is_alive!",
                target_id,
                game_id as "game_id!",
                NULL as "target_name: _"
            FROM players
            WHERE auth_token = ?
            "#,
            killer_token
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query for killer: {}", e);
            AppError::InternalServerError
        })?
        .ok_or(AppError::Unauthorized)
    }

    async fn get_player_by_secret_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        target_secret: &Uuid,
    ) -> Result<Player, AppError> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT
                id as "id!",
                name as "name!",
                secret_code as "secret_code!: Uuid",
                auth_token as "auth_token!",
                is_alive as "is_alive!",
                target_id,
                game_id as "game_id!",
                NULL as "target_name: _"
            FROM players
            WHERE secret_code = ?
            "#,
            target_secret
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to query for target: {}", e);
            AppError::InternalServerError
        })?
        .ok_or(AppError::NotFound(
            "Target secret does not correspond to an active player.".to_string(),
        ))
    }

    async fn get_game_by_id_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        game_id: i64,
    ) -> Result<Game, AppError> {
        sqlx::query_as!(
            Game,
            r#"
            SELECT id as "id!", code, status as "status: _", host_id, winner_id, created_at
            FROM games
            WHERE id = ?
            "#,
            game_id
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|_| AppError::InternalServerError)
    }

    fn validate_kill(killer: &Player, target: &Player, game: &Game) -> Result<(), AppError> {
        if !killer.is_alive {
            return Err(AppError::Forbidden(
                "A dead player cannot perform a kill.".to_string(),
            ));
        }

        if !target.is_alive {
            return Err(AppError::Forbidden(
                "The target is already dead.".to_string(),
            ));
        }

        if killer.game_id != target.game_id {
            return Err(AppError::Forbidden(
                "Killer and target are not in the same game.".to_string(),
            ));
        }

        if killer.id == target.id {
            return Err(AppError::Forbidden(
                "A player cannot kill themselves.".to_string(),
            ));
        }

        if game.status != GameStatus::InProgress {
            return Err(AppError::UnprocessableEntity(
                "Game is not in progress.".to_string(),
            ));
        }

        if killer.target_id != Some(target.id) {
            return Err(AppError::Forbidden(
                "The identified target is not the killer's current target.".to_string(),
            ));
        }

        Ok(())
    }

    async fn update_game_state_after_kill(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        killer: &Player,
        target: &Player,
    ) -> Result<Option<String>, AppError> {
        sqlx::query!(
            "UPDATE players SET is_alive = FALSE WHERE id = ?",
            target.id
        )
        .execute(&mut **tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        let is_game_over = target.target_id == Some(killer.id) || target.target_id.is_none();
        let new_target_id = if is_game_over { None } else { target.target_id };

        sqlx::query!(
            "UPDATE players SET target_id = ? WHERE id = ?",
            new_target_id,
            killer.id
        )
        .execute(&mut **tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        if is_game_over {
            sqlx::query!(
                "UPDATE games SET status = 'finished', winner_id = ? WHERE id = ?",
                killer.id,
                killer.game_id
            )
            .execute(&mut **tx)
            .await
            .map_err(|_| AppError::InternalServerError)?;
            Ok(None)
        } else {
            let new_target_name = if let Some(new_target_id) = target.target_id {
                sqlx::query_scalar!("SELECT name FROM players WHERE id = ?", new_target_id)
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(|_| AppError::InternalServerError)?
            } else {
                None
            };
            Ok(new_target_name)
        }
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

    pub async fn get_player_by_name(
        &self,
        game_id: i64,
        player_name: &str,
    ) -> Result<Option<Player>, AppError> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT
                id as "id!",
                name as "name!",
                secret_code as "secret_code!: Uuid",
                auth_token as "auth_token!",
                is_alive as "is_alive!",
                target_id,
                game_id as "game_id!",
                NULL as "target_name: _"
            FROM players
            WHERE game_id = ? AND name = ?
            "#,
            game_id,
            player_name
        )
        .fetch_optional(&self.0)
        .await
        .map_err(|_| AppError::InternalServerError)
    }

    pub async fn get_game_by_code(&self, code: &str) -> Result<Option<Game>, AppError> {
        sqlx::query_as!(
            Game,
            r#"
            SELECT id as "id!", code, status as "status: _", host_id, winner_id, created_at
            FROM games
            WHERE code = ?
            "#,
            code
        )
        .fetch_optional(&self.0)
        .await
        .map_err(|_| AppError::InternalServerError)
    }

    pub async fn get_player_by_id(&self, player_id: i64) -> Result<Option<Player>, AppError> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT
                id as "id!",
                name as "name!",
                secret_code as "secret_code!: Uuid",
                auth_token as "auth_token!",
                is_alive as "is_alive!",
                target_id,
                game_id as "game_id!",
                NULL as "target_name: _"
            FROM players
            WHERE id = ?
            "#,
            player_id
        )
        .fetch_optional(&self.0)
        .await
        .map_err(|_| AppError::InternalServerError)
    }
}

impl Deref for Db {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
