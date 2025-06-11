use crate::{
    errors::AppError,
    models::{Game, GameStatus, Player},
};
use rand::seq::SliceRandom;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{env, ops::Deref};
use tracing::{debug, info};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Db(SqlitePool);

impl Db {
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

        Ok(Db(pool))
    }

    pub async fn create_game(
        &self,
        player_name: String,
        game_code: String,
    ) -> Result<(i64, i64, Uuid, String), sqlx::Error> {
        info!(
            "Creating game with code {} for player {}",
            game_code, player_name
        );
        let mut tx = self.0.begin().await?;
        debug!("Transaction started for create_game");

        let player_secret = Uuid::new_v4();
        let auth_token = Uuid::new_v4().to_string();

        let game_id = sqlx::query!(
            r#"
            INSERT INTO games (code, status)
            VALUES (?, 'lobby')
            RETURNING id
            "#,
            game_code
        )
        .fetch_one(&mut *tx)
        .await?
        .id;
        debug!("Game created with id: {}", game_id);

        let player_id = sqlx::query!(
            r#"
            INSERT INTO players (game_id, name, secret_code, auth_token)
            VALUES (?, ?, ?, ?)
            "#,
            game_id,
            player_name,
            player_secret,
            auth_token
        )
        .execute(&mut *tx)
        .await?
        .last_insert_rowid();
        debug!("Player created with id: {}", player_id);

        sqlx::query!(
            r#"
            UPDATE games
            SET host_id = ?
            WHERE id = ?
            "#,
            player_id,
            game_id
        )
        .execute(&mut *tx)
        .await?;
        debug!("Game host updated for game_id: {}", game_id);

        tx.commit().await?;
        debug!("Transaction committed for create_game");

        Ok((game_id, player_id, player_secret, auth_token))
    }

    pub async fn join_game(
        &self,
        game_code: String,
        player_name: String,
    ) -> Result<(i64, i64, Uuid, String), AppError> {
        info!(
            "Player {} joining game with code {}",
            player_name, game_code
        );
        let mut tx = self.0.begin().await.map_err(|e| {
            tracing::error!("Failed to begin transaction: {}", e);
            AppError::InternalServerError
        })?;
        debug!("Transaction started for join_game");

        let game = sqlx::query_as!(
            Game,
            r#"
            SELECT
                id as "id!",
                status as "status: _",
                host_id as "host_id: _"
            FROM games
            WHERE code = ?
            "#,
            game_code
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or(AppError::NotFound("Game not found".to_string()))?;
        debug!("Fetched game: {:?}", game);

        if game.status != GameStatus::Lobby {
            debug!(
                "Attempted to join game with status {:?}, expected Lobby",
                game.status
            );
            return Err(AppError::UnprocessableEntity(
                "You can not join games that aren't in lobby".to_string(),
            ));
        }

        if let Some(player) = self
            .get_player_by_name(game.id, &player_name)
            .await
            .map_err(|_| AppError::InternalServerError)?
        {
            debug!("Player {} found in game {}", player_name, game_code);
            if player.is_alive {
                // This is a reconnection
                debug!("Player {} is alive, rejoining.", player_name);
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
                debug!("Player {} is dead.", player_name);
                return Err(AppError::Forbidden(
                    "You have been eliminated from this game.".to_string(),
                ));
            }
        }

        // New player
        debug!("Player {} is a new player.", player_name);
        let player_secret = Uuid::new_v4();
        let auth_token = Uuid::new_v4().to_string();

        let player_id = sqlx::query!(
            r#"
            INSERT INTO players (game_id, name, secret_code, auth_token)
            VALUES (?, ?, ?, ?)
            "#,
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
        debug!("New player {} created with id {}", player_name, player_id);

        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            AppError::InternalServerError
        })?;
        debug!("Transaction committed for join_game");

        Ok((game.id, player_id, player_secret, auth_token))
    }

    pub async fn get_players_by_game_id(&self, game_id: i64) -> Result<Vec<Player>, sqlx::Error> {
        info!("Fetching players for game_id: {}", game_id);
        let players = sqlx::query_as!(
            Player,
            r#"
            SELECT
                p.id as "id!",
                p.name,
                p.secret_code as "secret_code: _",
                p.auth_token,
                p.is_alive,
                p.target_id,
                p.game_id,
                t.name as "target_name: _"
            FROM players p
            LEFT JOIN players t ON p.target_id = t.id
            WHERE p.game_id = ?
            "#,
            game_id
        )
        .fetch_all(&self.0)
        .await?;
        debug!("Found {} players for game_id {}", players.len(), game_id);
        Ok(players)
    }

    pub async fn get_player_by_auth_token(
        &self,
        auth_token: &str,
    ) -> Result<Option<Player>, sqlx::Error> {
        info!("Fetching player by auth_token");
        let player = sqlx::query_as!(
            Player,
            r#"
            SELECT
                id as "id!",
                name,
                secret_code as "secret_code: _",
                auth_token,
                is_alive,
                target_id,
                game_id,
                NULL as "target_name: _"
            FROM players
            WHERE auth_token = ?
            "#,
            auth_token
        )
        .fetch_optional(&self.0)
        .await?;
        debug!("Found player: {:?}", player);
        Ok(player)
    }

    pub async fn start_game(
        &self,
        game_code: &str,
        player_id: i64,
    ) -> Result<Vec<Player>, AppError> {
        info!("Starting game with code {}", game_code);
        let mut tx = self
            .0
            .begin()
            .await
            .map_err(|_| AppError::InternalServerError)?;
        debug!("Transaction started for start_game");

        let game = sqlx::query_as!(
            Game,
            r#"
            SELECT
                id as "id!",
                status as "status: _",
                host_id as "host_id: _"
            FROM games
            WHERE code = ?
            "#,
            game_code
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or_else(|| AppError::NotFound("Game not found.".to_string()))?;
        debug!("Found game: {:?}", game);

        if game.host_id != Some(player_id) {
            debug!("Player {} is not the host of game {}", player_id, game_code);
            return Err(AppError::Forbidden(
                "Only the host can start the game.".to_string(),
            ));
        }

        if game.status != GameStatus::Lobby {
            debug!(
                "Attempted to start game with status {:?}, expected Lobby",
                game.status
            );
            return Err(AppError::UnprocessableEntity(
                "Game has already started.".to_string(),
            ));
        }

        let players = self
            .get_players_by_game_id(game.id)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        debug!("Found {} players for game {}", players.len(), game_code);

        if players.len() < 2 {
            debug!(
                "Attempted to start game with {} players, requires at least 2",
                players.len()
            );
            return Err(AppError::UnprocessableEntity(
                "You need at least 2 players to start the game.".to_string(),
            ));
        }

        let mut a_players = players.clone();
        a_players.shuffle(&mut rand::rng());
        let mut b_players = a_players.clone();
        b_players.rotate_left(1);

        for (player, target) in a_players.iter().zip(b_players.iter()) {
            sqlx::query!(
                r#"
                UPDATE players
                SET target_id = ?
                WHERE id = ?
                "#,
                target.id,
                player.id
            )
            .execute(&mut *tx)
            .await
            .map_err(|_| AppError::InternalServerError)?;
            debug!("Assigned target {} to player {}", target.id, player.id);
        }

        sqlx::query!(
            r#"
            UPDATE games
            SET status = 'inprogress'
            WHERE id = ?
            "#,
            game.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;
        info!("Game {} started", game_code);

        tx.commit()
            .await
            .map_err(|_| AppError::InternalServerError)?;
        debug!("Transaction committed for start_game");

        self.get_players_by_game_id(game.id).await.map_err(|e| {
            tracing::error!("Failed to get players by game id: {}", e);
            AppError::InternalServerError
        })
    }

    pub async fn process_kill(
        &self,
        game_code: &str,
        killer_token: &str,
        target_secret: &Uuid,
    ) -> Result<(i64, String, String, Option<String>), AppError> {
        info!(
            "Processing kill in game {} for target with secret {}",
            game_code, target_secret
        );
        let mut tx = self
            .0
            .begin()
            .await
            .map_err(|_| AppError::InternalServerError)?;
        debug!("Transaction started for process_kill");

        let game = self.get_game_by_code_in_tx(&mut tx, game_code).await?;
        debug!("Found game: {:?}", game);
        let killer = self
            .get_player_by_auth_token_in_tx(&mut tx, killer_token, game.id)
            .await?;
        debug!("Found killer: {:?}", killer);
        let target = self
            .get_player_by_secret_in_tx(&mut tx, target_secret, game.id)
            .await?;
        debug!("Found target: {:?}", target);

        Self::validate_kill(&killer, &target, &game)?;
        debug!("Kill validated");

        let new_target_name = self
            .update_game_state_after_kill(&mut tx, &killer, &target)
            .await?;
        debug!("Game state updated. New target: {:?}", new_target_name);

        tx.commit()
            .await
            .map_err(|_| AppError::InternalServerError)?;
        debug!("Transaction committed for process_kill");

        Ok((killer.id, killer.name, target.name, new_target_name))
    }

    async fn get_player_by_auth_token_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        killer_token: &str,
        game_id: i64,
    ) -> Result<Player, AppError> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT
                id as "id!",
                name,
                secret_code as "secret_code: _",
                auth_token,
                is_alive,
                target_id,
                game_id,
                NULL as "target_name: _"
            FROM players
            WHERE auth_token = ? AND game_id = ?
            "#,
            killer_token,
            game_id
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
        game_id: i64,
    ) -> Result<Player, AppError> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT
                id as "id!",
                name,
                secret_code as "secret_code: _",
                auth_token,
                is_alive,
                target_id,
                game_id,
                NULL as "target_name: _"
            FROM players
            WHERE secret_code = ? AND game_id = ?
            "#,
            target_secret,
            game_id
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

    async fn get_game_by_code_in_tx(
        &self,
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        game_code: &str,
    ) -> Result<Game, AppError> {
        sqlx::query_as!(
            Game,
            r#"
            SELECT
                id as "id!",
                status as "status: _",
                host_id as "host_id: _"
            FROM games
            WHERE code = ?
            "#,
            game_code
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or_else(|| AppError::NotFound("Game not found".to_string()))
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
        // Mark target as dead
        sqlx::query!(
            r#"
            UPDATE players
            SET is_alive = FALSE
            WHERE id = ?
            "#,
            target.id
        )
        .execute(&mut **tx)
        .await?;

        // Check how many players are left alive *within this transaction*
        let alive_count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)
            FROM players
            WHERE game_id = ? AND is_alive = TRUE
            "#,
            killer.game_id
        )
        .fetch_one(&mut **tx)
        .await?;

        // Is the game over?
        if alive_count <= 1 {
            // Update game status to finished and set winner
            sqlx::query!(
                r#"
                UPDATE games
                SET status = 'finished', winner_id = ?
                WHERE id = ?
                "#,
                killer.id,
                killer.game_id
            )
            .execute(&mut **tx)
            .await?;

            // The winner has no more targets
            sqlx::query!(
                r#"
                UPDATE players
                SET target_id = NULL
                WHERE id = ?
                "#,
                killer.id
            )
            .execute(&mut **tx)
            .await?;

            Ok(None) // No new target
        } else {
            // Reassign target for the killer
            let new_target_id = target.target_id;
            sqlx::query!(
                r#"
                UPDATE players
                SET target_id = ?
                WHERE id = ?
                "#,
                new_target_id,
                killer.id,
            )
            .execute(&mut **tx)
            .await?;

            let new_target_name = sqlx::query!(
                r#"
                SELECT name
                FROM players
                WHERE id = ?
                "#,
                new_target_id
            )
            .fetch_one(&mut **tx)
            .await?
            .name;

            Ok(Some(new_target_name))
        }
    }

    pub async fn get_player_by_name(
        &self,
        game_id: i64,
        player_name: &str,
    ) -> Result<Option<Player>, AppError> {
        info!(
            "Fetching player by name {} for game_id {}",
            player_name, game_id
        );
        let player = sqlx::query_as!(
            Player,
            r#"
            SELECT
                id as "id!",
                name,
                secret_code as "secret_code: _",
                auth_token,
                is_alive,
                target_id,
                game_id,
                NULL as "target_name: _"
            FROM players
            WHERE game_id = ? AND name = ?
            "#,
            game_id,
            player_name
        )
        .fetch_optional(&self.0)
        .await
        .map_err(|e| {
            tracing::error!("Failed to get player by name: {}", e);
            AppError::InternalServerError
        })?;
        debug!("Found player: {:?}", player);
        Ok(player)
    }

    pub async fn get_game_by_code(&self, code: &str) -> Result<Option<Game>, AppError> {
        info!("Fetching game by code: {}", code);
        let game = sqlx::query_as!(
            Game,
            r#"
            SELECT
                id as "id!",
                status as "status: _",
                host_id as "host_id: _"
            FROM games
            WHERE code = ?
            "#,
            code
        )
        .fetch_optional(&self.0)
        .await
        .map_err(|_| AppError::InternalServerError)?;
        debug!("Found game: {:?}", game);
        Ok(game)
    }
}

impl Deref for Db {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
