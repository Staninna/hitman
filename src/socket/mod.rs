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
        move |socket, payload| {
            tokio::spawn(handlers::create_game(socket, payload, state));
        }
    });

    socket.on("join_game", {
        let state = state.clone();
        move |socket, payload| {
            tokio::spawn(handlers::join_game(socket, payload, state));
        }
    });

    socket.on("start_game", {
        let state = state.clone();
        move |socket, payload| {
            tokio::spawn(handlers::start_game(socket, payload, state));
        }
    });
}
