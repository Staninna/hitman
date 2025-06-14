use crate::db::Db;
use crate::errors::AppError;
use crate::models::{Game, GameStatus, Player};
use sqlx;
use tracing::debug;

impl Db {
    pub async fn process_kill(
        &self,
        game_code: &str,
        killer_token: &str,
        target_secret: &str,
    ) -> Result<(i64, String, String, Option<String>), AppError> {
        let mut tx = self
            .0
            .begin()
            .await
            .map_err(|_| AppError::InternalServerError)?;
        debug!("Transaction started for process_kill");

        let game = self.get_game_by_code_in_tx(&mut tx, game_code).await?;
        let killer = self
            .get_player_by_auth_token_in_tx(&mut tx, killer_token, game.id)
            .await?;
        let target = self
            .get_player_by_secret_in_tx(&mut tx, target_secret, game.id)
            .await?;

        Self::validate_kill(&killer, &target, &game)?;

        let new_target_name = self
            .update_game_state_after_kill(&mut tx, &killer, &target)
            .await?;
        tx.commit()
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok((killer.id, killer.name, target.name, new_target_name))
    }

    fn validate_kill(killer: &Player, target: &Player, game: &Game) -> Result<(), AppError> {
        if !killer.is_alive {
            return Err(AppError::Forbidden(
                "A dead player cannot perform a kill.".into(),
            ));
        }
        if !target.is_alive {
            return Err(AppError::Forbidden("The target is already dead.".into()));
        }
        if killer.game_id != target.game_id {
            return Err(AppError::Forbidden(
                "Killer and target are not in the same game.".into(),
            ));
        }
        if killer.id == target.id {
            return Err(AppError::Forbidden(
                "A player cannot kill themselves.".into(),
            ));
        }
        if game.status != GameStatus::InProgress {
            return Err(AppError::UnprocessableEntity(
                "Game is not in progress.".into(),
            ));
        }
        if killer.target_id != Some(target.id) {
            return Err(AppError::Forbidden(
                "The identified target is not the killer's current target.".into(),
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
        .await?;

        let alive_count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM players WHERE game_id = ? AND is_alive = TRUE",
            killer.game_id
        )
        .fetch_one(&mut **tx)
        .await?;
        if alive_count <= 1 {
            sqlx::query!(
                "UPDATE games SET status = 'finished', winner_id = ? WHERE id = ?",
                killer.id,
                killer.game_id
            )
            .execute(&mut **tx)
            .await?;
            sqlx::query!(
                "UPDATE players SET target_id = NULL WHERE id = ?",
                killer.id
            )
            .execute(&mut **tx)
            .await?;
            Ok(None)
        } else {
            let new_target_id = target.target_id;
            sqlx::query!(
                "UPDATE players SET target_id = ? WHERE id = ?",
                new_target_id,
                killer.id
            )
            .execute(&mut **tx)
            .await?;
            let new_target_name =
                sqlx::query!("SELECT name FROM players WHERE id = ?", new_target_id)
                    .fetch_one(&mut **tx)
                    .await?
                    .name;
            Ok(Some(new_target_name))
        }
    }
}
