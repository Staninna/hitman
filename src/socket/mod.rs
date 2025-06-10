use crate::state::AppState;
use socketioxide::extract::{SocketRef};
use tracing::info;

mod handlers;
mod utils;

pub fn on_connect(socket: SocketRef, state: AppState) {
    info!(
        "Socket connected: {} with transport {:?}",
        socket.id,
        socket.transport_type()
    );

    let state_for_create = state.clone();
    socket.on(
        "create_game",
        move |socket, payload, ack| {
            let state = state_for_create.clone();
            tokio::spawn(async move {
                handlers::create_game(socket, payload, ack, state).await;
            });
        },
    );

    let state_for_join = state.clone();
    socket.on(
        "join_game",
        move |socket, payload, ack| {
            let state = state_for_join.clone();
            tokio::spawn(async move {
                handlers::join_game(socket, payload, ack, state).await;
            });
        },
    );

    socket.on(
        "start_game",
        move |socket, payload| {
            let state = state.clone();
            tokio::spawn(async move {
                handlers::start_game(socket, payload, state).await;
            });
        },
    );
} 