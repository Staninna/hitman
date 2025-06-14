use sqlx::{Executor, Postgres};

use super::Db;
use crate::errors::AppError;
use crate::models::{GameStatus, Player};
use tracing::{debug, info};

impl Db {
    // -------- Public player APIs ---------

    pub async fn get_players_by_game_id<'e, E>(
        &self,
        executor: E,
        game_id: i32,
    ) -> Result<Vec<Player>, sqlx::Error>
    where
        E: Executor<'e, Database = Postgres>,
    {
        info!("Fetching players for game_id: {}", game_id);
        let players = sqlx::query_as!(
            Player,
            r#"
            SELECT
                p.id as "id!",
                p.name,
                p.secret_code,
                p.auth_token,
                p.is_alive,
                p.target_id,
                p.game_id,
                COALESCE(t.name, '') as "target_name: _"
            FROM players p
            LEFT JOIN players t ON p.target_id = t.id
            WHERE p.game_id = $1
            "#,
            game_id
        )
        .fetch_all(executor)
        .await?;
        debug!("Found {} players for game_id {}", players.len(), game_id);
        Ok(players)
    }

    pub async fn get_player_by_auth_token(
        &self,
        auth_token: &str,
    ) -> Result<Option<Player>, sqlx::Error> {
        info!("Fetching player by auth_token: {}", auth_token);

        let player = sqlx::query_as!(
            Player,
            r#"
            SELECT
                p.id as "id!",
                p.name,
                p.secret_code,
                p.auth_token,
                p.is_alive,
                p.target_id,
                p.game_id,
                COALESCE(t.name, '') as "target_name: _"
            FROM players p
            LEFT JOIN players t ON p.target_id = t.id
            WHERE p.auth_token = $1
            "#,
            auth_token
        )
        .fetch_optional(&self.0)
        .await?;

        Ok(player)
    }

    pub async fn get_player_by_name(
        &self,
        game_id: i32,
        player_name: &str,
    ) -> Result<Option<Player>, AppError> {
        info!(
            "Fetching player by name {} for game_id {}",
            player_name, game_id
        );
        let normalised_name = crate::utils::normalise_name(player_name);
        let player = sqlx::query_as!(
            Player,
            r#"
            SELECT
                p.id as "id!",
                p.name,
                p.secret_code,
                p.auth_token,
                p.is_alive,
                p.target_id,
                p.game_id,
                COALESCE(t.name, '') as "target_name: _"
            FROM players p
            LEFT JOIN players t ON p.target_id = t.id
            WHERE p.game_id = $1 AND LOWER(p.name) = $2
            "#,
            game_id,
            normalised_name
        )
        .fetch_optional(&self.0)
        .await
        .map_err(|e| {
            tracing::warn!(game_id, player_name, "Failed to get player by name: {}", e);
            AppError::InternalServerError
        })?;
        debug!("Found player: {:?}", player);
        Ok(player)
    }

    pub async fn leave_game(&self, game_code: &str, auth_token: &str) -> Result<(), AppError> {
        info!(
            "Player with token {} leaving game {}",
            auth_token, game_code
        );
        let mut tx = self.0.begin().await.map_err(|e| {
            tracing::warn!(game_code, auth_token, "Failed to begin transaction: {}", e);
            AppError::InternalServerError
        })?;

        let game = self.get_game_by_code_in_tx(&mut tx, game_code).await?;
        let player = self
            .get_player_by_auth_token_in_tx(&mut tx, auth_token, game.id)
            .await?;

        if game.status == GameStatus::Lobby {
            let is_host = game.host_id == Some(player.id);

            sqlx::query!("DELETE FROM players WHERE id = $1", player.id)
                .execute(&mut *tx)
                .await
                .map_err(|_| AppError::InternalServerError)?;

            if is_host {
                let remaining_players: Vec<Player> = sqlx::query_as!(
                    Player,
                    r#"
                    SELECT
                        p.id as "id!",
                        p.name,
                        p.secret_code,
                        p.auth_token,
                        p.is_alive,
                        p.target_id,
                        p.game_id,
                        COALESCE(t.name, '') as "target_name: _"
                    FROM players p
                    LEFT JOIN players t ON p.target_id = t.id
                    WHERE p.game_id = $1 ORDER BY p.id ASC
                    "#,
                    game.id
                )
                .fetch_all(&mut *tx)
                .await
                .map_err(|_| AppError::InternalServerError)?;

                if remaining_players.is_empty() {
                    // Last player (the host) left, delete the game
                    sqlx::query!("DELETE FROM games WHERE id = $1", game.id)
                        .execute(&mut *tx)
                        .await
                        .map_err(|_| AppError::InternalServerError)?;
                } else {
                    // Assign a new host (the one who joined earliest)
                    let new_host_id = remaining_players[0].id;
                    sqlx::query!(
                        "UPDATE games SET host_id = $1 WHERE id = $2",
                        new_host_id,
                        game.id
                    )
                    .execute(&mut *tx)
                    .await
                    .map_err(|_| AppError::InternalServerError)?;
                }
            }
        } else {
            // If game is in progress or finished, just mark as not alive
            sqlx::query!(
                "UPDATE players SET is_alive = false WHERE id = $1",
                player.id
            )
            .execute(&mut *tx)
            .await
            .map_err(|_| AppError::InternalServerError)?;
        }

        tx.commit()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    // ---------- Private helpers (within transaction) -------------

    pub(crate) async fn get_player_by_auth_token_in_tx<'a>(
        &self,
        tx: &mut sqlx::Transaction<'a, sqlx::Postgres>,
        killer_token: &str,
        game_id: i32,
    ) -> Result<Player, AppError> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT
                p.id as "id!",
                p.name,
                p.secret_code,
                p.auth_token,
                p.is_alive,
                p.target_id,
                p.game_id,
                COALESCE(t.name, '') as "target_name: _"
            FROM players p
            LEFT JOIN players t ON p.target_id = t.id
            WHERE p.auth_token = $1 AND p.game_id = $2
            "#,
            killer_token,
            game_id
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| {
            tracing::warn!(
                killer_token,
                game_id,
                "Failed to query for killer by auth token: {}",
                e
            );
            AppError::InternalServerError
        })?
        .ok_or(AppError::Unauthorized)
    }

    pub(crate) async fn get_player_by_secret_in_tx<'a>(
        &self,
        tx: &mut sqlx::Transaction<'a, sqlx::Postgres>,
        target_secret: &str,
        game_id: i32,
    ) -> Result<Player, AppError> {
        sqlx::query_as!(
            Player,
            r#"
            SELECT
                p.id as "id!",
                p.name,
                p.secret_code,
                p.auth_token,
                p.is_alive,
                p.target_id,
                p.game_id,
                COALESCE(t.name, '') as "target_name: _"
            FROM players p
            LEFT JOIN players t ON p.target_id = t.id
            WHERE p.secret_code = $1 AND p.game_id = $2
            "#,
            target_secret,
            game_id
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| {
            tracing::warn!(
                target_secret,
                game_id,
                "Failed to query for target by secret: {}",
                e
            );
            AppError::InternalServerError
        })?
        .ok_or(AppError::NotFound(
            "Target secret does not correspond to an active player.".to_string(),
        ))
    }
}
