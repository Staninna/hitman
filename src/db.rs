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
    ) -> Result<Option<(i64, i64, Uuid, String)>, sqlx::Error> {
        let mut tx = self.0.begin().await?;

        let game_row = sqlx::query!(
            r#"
            SELECT 
                id, 
                code, 
                status, 
                host_id, 
                winner_id,
                created_at
            FROM games 
            WHERE code = ?
            "#,
            game_code
        )
        .fetch_optional(&mut *tx)
        .await?;

        if game_row.is_none() {
            return Ok(None);
        }

        let row = game_row.unwrap();

        let game_status: GameStatus = match row.status.as_str() {
            "lobby" => GameStatus::Lobby,
            "in_progress" => GameStatus::InProgress,
            "finished" => GameStatus::Finished,
            _ => return Ok(None),
        };

        let game = Game {
            id: row.id.unwrap(),
            code: row.code,
            status: game_status,
            host_id: row.host_id,
            winner_id: row.winner_id,
            created_at: row.created_at,
        };

        if game.status != GameStatus::Lobby {
            return Ok(None); // TODO: Return error u can not join games that aren't in lobby
        }

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
        .await?
        .last_insert_rowid();

        tx.commit().await?;

        Ok(Some((game.id, player_id, player_secret, auth_token)))
    }

    pub async fn get_players_by_game_id(&self, game_id: i64) -> Result<Vec<Player>, sqlx::Error> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT id, name, secret_code as "secret_code: _", auth_token, is_alive, target_id, game_id, NULL as "target_name: _"
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
        let row = sqlx::query!(
            r#"
            SELECT id, name, secret_code, auth_token, is_alive, target_id, game_id
            FROM players
            WHERE auth_token = ?
            "#,
            auth_token
        )
        .fetch_optional(&self.0)
        .await?;

        Ok(row.map(|r| Player {
            id: r.id.unwrap(),
            name: r.name,
            secret_code: r.secret_code.parse().unwrap(),
            auth_token: r.auth_token,
            is_alive: r.is_alive,
            target_id: r.target_id,
            game_id: r.game_id,
            target_name: None,
        }))
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

        let game_row = sqlx::query!(
            "SELECT id, code, status, host_id, winner_id, created_at FROM games WHERE code = ?",
            game_code
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or_else(|| AppError::NotFound("Game not found.".to_string()))?;

        let game_status: GameStatus = match game_row.status.as_str() {
            "lobby" => GameStatus::Lobby,
            "in_progress" => GameStatus::InProgress,
            "finished" => GameStatus::Finished,
            _ => {
                return Err(AppError::InternalServerError);
            }
        };

        let game = Game {
            id: game_row.id.unwrap(),
            code: game_row.code,
            status: game_status,
            host_id: game_row.host_id,
            winner_id: game_row.winner_id,
            created_at: game_row.created_at,
        };

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

        // TODO: Players cant target themselves and multiple players cant have the same target and 2 players cant target each other
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
                // TODO: wtf we do here?
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
    ) -> Result<(String, String, Option<String>), AppError> {
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

        Ok((killer.name, target.name, new_target_name))
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
            SELECT id, code, status as "status: _", host_id, winner_id, created_at
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
}

impl Deref for Db {
    type Target = SqlitePool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
