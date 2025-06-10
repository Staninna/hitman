use socketioxide::extract::{AckSender, Data, SocketRef, State};
use tracing::info;

use crate::state::AppState;

pub fn on_connect(socket: SocketRef, State(state): State<AppState>) {
    info!(
        "Socket connected: {} with transport {:?}",
        socket.id,
        socket.transport_type()
    );

    let state_for_create = state.clone();
    socket.on(
        "create_game",
        move |socket: SocketRef, Data::<serde_json::Value>(payload), _ack: AckSender| {
            async move {
                info!(
                    "[socket {}] Received create_game: {:?}, state: {:?}",
                    socket.id,
                    payload,
                    state_for_create.db_pool
                );
                // TODO: Implement game creation logic
            }
        },
    );

    let state_for_join = state.clone();
    socket.on(
        "join_game",
        move |socket: SocketRef, Data::<serde_json::Value>(payload), _ack: AckSender| {
            async move {
                info!(
                    "[socket {}] Received join_game: {:?}, state: {:?}",
                    socket.id,
                    payload,
                    state_for_join.db_pool
                );
                // TODO: Implement game joining logic
            }
        },
    );

    socket.on(
        "start_game",
        move |socket: SocketRef, Data::<serde_json::Value>(payload)| {
            async move {
                info!(
                    "[socket {}] Received start_game: {:?}, state: {:?}",
                    socket.id,
                    payload,
                    state.db_pool
                );
                // TODO: Implement game starting logic
            }
        },
    );
} 