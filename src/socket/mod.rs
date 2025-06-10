use crate::state::AppState;
use socketioxide::extract::SocketRef;
use tracing::info;

mod handlers;

pub fn on_connect(socket: SocketRef, state: AppState) {
    info!(
        "Socket connected: {} with transport {:?}",
        socket.id,
        socket.transport_type()
    );

    socket.on("create_game", {
        let state = state.clone();
        move |socket, payload, ack| {
            let state = state.clone();
            tokio::spawn(async move {
                handlers::create_game(socket, payload, ack, state).await;
            });
        }
    });

    socket.on("join_game", {
        let state = state.clone();
        move |socket, payload, ack| {
            let state = state.clone();
            tokio::spawn(async move {
                handlers::join_game(socket, payload, ack, state).await;
            });
        }
    });

    socket.on("start_game", {
        let state = state.clone();
        move |socket, payload| {
            let state = state.clone();
            tokio::spawn(async move {
                handlers::start_game(socket, payload, state).await;
            });
        }
    });
}
