use crate::state::AppState;
use socketioxide::extract::SocketRef;
use socketioxide::socket::DisconnectReason;
use tracing::info;

use crate::socket::handlers::handle_disconnect;

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

    socket.on_disconnect({
        let state = state.clone();
        move |socket: SocketRef, reason: DisconnectReason| {
            info!("Socket disconnected: {} with reason: {}", socket.id, reason);
            tokio::spawn(handle_disconnect(socket, state));
        }
    });
}
