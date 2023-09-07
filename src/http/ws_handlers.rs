use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{ws::WebSocket, ConnectInfo, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
};
use bytemuck::try_from_bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message as TMessage;

use crate::primitives::{block::Block, Chain};

use super::{SocketConnectionType, WSState};

fn handle_new_block(block: Vec<u8>, chain: &mut Chain) {
    match try_from_bytes::<Block>(&block) {
        Ok(block) => {
            let b_idx = block.index;
            if chain.push_block(*block) {
                tracing::info!("RECEIVED BLOCK: {b_idx}")
            } else {
                tracing::warn!("RECEIVED INVALID BLOCK: {b_idx}")
            }
        }
        Err(err) => tracing::error!("BAD PAYLOAD: {err}"),
    }
}

pub async fn peers(State(state): State<WSState>) -> impl IntoResponse {
    axum_streams::StreamBodyAs::json_array(
        futures_util::stream::iter(state.connections.lock().await.clone()).map(|x| x.0),
    )
}

pub async fn add_peer(
    State(state): State<WSState>,
    Query(peer): Query<SocketAddr>,
) -> impl IntoResponse {
    if state
        .connections
        .lock()
        .await
        .insert(peer, SocketConnectionType::Outcome)
        .is_none()
    {
        tokio::spawn(async move {
            let (socket_stream, _response) = tokio_tungstenite::connect_async(peer.to_string())
                .await
                .unwrap();
            let socket_stream = Arc::new(Mutex::new(socket_stream));

            let ss = socket_stream.clone();
            tokio::spawn(async move {
                let mut receiver = state.app_state.sockets.subscribe();
                while let Ok(message) = receiver.recv().await {
                    ss.lock()
                        .await
                        .send(TMessage::Binary(message.into_data()))
                        .await
                        .unwrap();
                }
            });

            while let Some(Ok(message)) = socket_stream.lock().await.next().await {
                let block = message.into_data();
                handle_new_block(block, &mut state.app_state.chain.write().await as _);
            }
        });

        return StatusCode::OK;
    }

    StatusCode::CONFLICT
}

pub async fn ws_index(
    ws: WebSocketUpgrade,
    state: State<WSState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    state
        .0
        .connections
        .lock()
        .await
        .insert(addr, SocketConnectionType::Income);

    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(socket: WebSocket, _who: SocketAddr, State(state): State<WSState>) {
    let socket = Arc::new(Mutex::new(socket));
    let mut rx = state.app_state.sockets.subscribe();

    let s = socket.clone();
    tokio::spawn(async move {
        while let Ok(message) = rx.recv().await {
            if let Err(err) = s.lock().await.send(message).await {
                tracing::error!("COULD NOT SEND MESSAGE {err}");
            }
        }
    });

    while let Some(Ok(message)) = socket.lock().await.recv().await {
        let block = message.into_data();
        handle_new_block(block, &mut state.app_state.chain.write().await as _);
    }
}
