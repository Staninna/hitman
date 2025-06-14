use super::super::Db;
use crate::errors::AppError;
use crate::models::{GameStatus, Player};
use crate::utils::generate_code;
use rand::seq::SliceRandom;
use tracing::{debug, info};
use uuid::Uuid;

impl Db {
    /// Create a new game and the first (host) player
    pub async fn create_game(
        &self,
        mut player_name: String,
        game_code: String,
    ) -> Result<(i32, i32, String, String), sqlx::Error> {
        player_name = player_name.trim().to_string();
        info!(
            "Creating game with code {} for player {}",
            game_code, player_name
        );
        let mut tx = self.0.begin().await?;
        debug!("Transaction started for create_game");

        let player_secret = generate_code(7);
        let auth_token = Uuid::new_v4().to_string();

        let game_id = sqlx::query!(
            "INSERT INTO games (code) VALUES ($1) RETURNING id",
            game_code
        )
        .fetch_one(&mut *tx)
        .await?
        .id;

        let player_id: i32 = sqlx::query_scalar!(
            "INSERT INTO players (game_id, name, secret_code, auth_token) VALUES ($1, $2, $3, $4) RETURNING id",
            game_id,
            player_name,
            player_secret,
            auth_token
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            "UPDATE games SET host_id = $1 WHERE id = $2",
            player_id,
            game_id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        Ok((game_id, player_id, player_secret, auth_token))
    }

    /// Existing or new player joins a lobby
    pub async fn join_game(
        &self,
        game_code: String,
        player_name: String,
    ) -> Result<(i32, i32, String, String), AppError> {
        let player_name = player_name.trim().to_string();
        info!("Player {} joining game {}", player_name, game_code);
        let mut tx = self
            .0
            .begin()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        let game = self.get_game_by_code_in_tx(&mut tx, &game_code).await?;
        if game.status != GameStatus::Lobby {
            return Err(AppError::UnprocessableEntity(
                "This game has already started or finished, so new players can no longer join.".to_string(),
            ));
        }

        // Check if a player with the requested name already exists in this game
        if let Some(p) = self.get_player_by_name(game.id, &player_name).await? {
            if p.is_alive {
                // A living player with this name is already in the lobby – reject the join attempt.
                return Err(AppError::UnprocessableEntity(
                    "That name is already being used by another player in this lobby. Please choose a different name.".to_string(),
                ));
            } else {
                // The player existed previously but has been eliminated – they cannot re-join.
                return Err(AppError::Forbidden(
                    "You were eliminated earlier in this game and cannot rejoin.".to_string(),
                ));
            }
        }

        // New player
        let player_secret = generate_code(7);
        let auth_token = Uuid::new_v4().to_string();

        let player_id: i32 = sqlx::query_scalar!(
            "INSERT INTO players (game_id, name, secret_code, auth_token) VALUES ($1, $2, $3, $4) RETURNING id",
            game.id,
            player_name,
            player_secret,
            auth_token
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return AppError::UnprocessableEntity("That name is already being used by another player in this lobby. Please choose a different name.".to_string());
                }
            }
            tracing::warn!(
                game_id = game.id,
                player_name,
                "Failed to insert player: {}",
                e
            );
            AppError::InternalServerError
        })?;

        tx.commit()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok((game.id, player_id, player_secret, auth_token))
    }

    /// Host starts the game – assigns targets and flips status
    pub async fn start_game(
        &self,
        game_code: &str,
        player_id: i32,
    ) -> Result<Vec<Player>, AppError> {
        info!("Starting game {} by player {}", game_code, player_id);
        let mut tx = self
            .0
            .begin()
            .await
            .map_err(|_| AppError::InternalServerError)?;
        let game = self.get_game_by_code_in_tx(&mut tx, game_code).await?;
        if game.host_id != Some(player_id) {
            return Err(AppError::Forbidden(
                "Only the host (the person who created the game) can start it.".to_string(),
            ));
        }

        let players = self.get_players_by_game_id(&mut *tx, game.id).await?;
        if players.len() < 2 {
            return Err(AppError::UnprocessableEntity(
                "You need at least 2 players in the lobby to start the game. Invite someone else to join first!".to_string(),
            ));
        }

        let mut ids: Vec<i32> = players.iter().map(|p| p.id).collect();
        ids.shuffle(&mut rand::rng());

        for (idx, &pid) in ids.iter().enumerate() {
            let target_id = ids[(idx + 1) % ids.len()];
            sqlx::query!(
                "UPDATE players SET target_id = $1 WHERE id = $2",
                target_id,
                pid
            )
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query!(
            "UPDATE games SET status = 'in_progress' WHERE id = $1",
            game.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(self.get_players_by_game_id(&self.0, game.id).await?)
    }
}
