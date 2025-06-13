use super::super::Db;
use crate::errors::AppError;
use crate::models::{Game, GameInfo, Player};

impl Db {
    // ------- public queries --------

    pub async fn get_all_games(&self) -> Result<Vec<GameInfo>, sqlx::Error> {
        let games = sqlx::query_as!(
            GameInfo,
            r#"SELECT g.code, g.status as "status: _", (SELECT COUNT(*) FROM players p WHERE p.game_id = g.id) as "player_count!" FROM games g"#
        )
        .fetch_all(&self.0)
        .await?;
        Ok(games)
    }

    pub async fn get_game_by_code(&self, code: &str) -> Result<Option<Game>, AppError> {
        let game = sqlx::query_as!(
            Game,
            r#"SELECT id as "id!", status as "status: _", host_id as "host_id: _", code as "code: _" FROM games WHERE code = ?"#,
            code
        )
        .fetch_optional(&self.0)
        .await
        .map_err(|_| AppError::InternalServerError)?;
        Ok(game)
    }

    pub async fn get_game_state(
        &self,
        game_code: &str,
    ) -> Result<Option<(Game, Vec<Player>)>, AppError> {
        let game = self.get_game_by_code(game_code).await?;
        if let Some(g) = game {
            let players = self.get_players_by_game_id(g.id).await?;
            Ok(Some((g, players)))
        } else {
            Ok(None)
        }
    }

    pub async fn get_game_by_id(&self, game_id: i64) -> Result<Option<Game>, sqlx::Error> {
        sqlx::query_as!(
            Game,
            r#"SELECT id as "id!", status as "status: _", host_id as "host_id: _", code as "code: _" FROM games WHERE id = ?"#,
            game_id
        )
        .fetch_optional(&self.0)
        .await
    }

    // ------- helpers within transaction --------
    pub(crate) async fn get_game_by_code_in_tx<'a>(
        &self,
        tx: &mut sqlx::Transaction<'a, sqlx::Sqlite>,
        game_code: &str,
    ) -> Result<Game, AppError> {
        sqlx::query_as!(
            Game,
            r#"SELECT id as "id!", status as "status: _", host_id as "host_id: _", code as "code: _" FROM games WHERE code = ?"#,
            game_code
        )
        .fetch_optional(&mut **tx)
        .await
        .map_err(|_| AppError::InternalServerError)?
        .ok_or_else(|| AppError::NotFound("Game not found".to_string()))
    }
}
