use super::Db;
use crate::errors::AppError;
use crate::models::Player;
use tracing::{debug, info};
use uuid::Uuid;

impl Db {
    // -------- Public player APIs ---------

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
        info!("Fetching player by auth_token: {}", auth_token);

        let player = sqlx::query_as!(
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
            WHERE p.auth_token = ?
            "#,
            auth_token
        )
        .fetch_optional(&self.0)
        .await?;

        Ok(player)
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

        sqlx::query!(
            "UPDATE players SET is_alive = false WHERE id = ?",
            player.id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| AppError::InternalServerError)?;

        tx.commit()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(())
    }

    // ---------- Private helpers (within transaction) -------------

    pub(crate) async fn get_player_by_auth_token_in_tx<'a>(
        &self,
        tx: &mut sqlx::Transaction<'a, sqlx::Sqlite>,
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
        tx: &mut sqlx::Transaction<'a, sqlx::Sqlite>,
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
            tracing::warn!(
                target_secret = target_secret.to_string(),
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
