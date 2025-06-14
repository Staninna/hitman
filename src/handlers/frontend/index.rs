use super::context::IndexContext;
use crate::state::AppState;
use axum::http::StatusCode;
use axum::{
    extract::State,
    response::{Html, IntoResponse},
};
use tera::Context;

pub async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let mut ctx = Context::new();
    let index_ctx = IndexContext {
        is_game_page: false,
        show_join_modal: false,
        is_rejoin_page: false,
        ..Default::default()
    };
    ctx.insert("ctx", &index_ctx);
    match state.tera.render("welcome.tera.html", &ctx) {
        Ok(s) => Html(s).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
