use std::ops::Range;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use bytemuck::bytes_of;
use futures_util::StreamExt;
use serde_json::json;

use crate::primitives::{blob::Blob, block::Block};

use super::{data_helpers::block_as_json, AppState};

pub async fn index() -> impl IntoResponse {
    "hello, world!"
}

pub async fn mine_block(
    State(s): State<AppState>,
    Query(data): Query<Vec<u8>>,
) -> impl IntoResponse {
    let block = s
        .chain
        .write()
        .await
        .generate_next_block(Blob::from(&data as &[_]));
    let block_idx = block.index;

    tokio::spawn(async move {
        while s.sockets.len() != 0 {}
        s.sockets
            .send(axum::extract::ws::Message::Binary(bytes_of(&block).into()))
            .unwrap();
    });

    Json(json!({ "block_index": block_idx }))
}

pub async fn blocks(State(s): State<AppState>) -> impl IntoResponse {
    let state = s.chain.read().await;
    let blocks = state.blocks() as *const [Block];

    // SAFETY: reading static data behind a lock,
    // there's no context where this operation is unsafe
    unsafe {
        axum_streams::StreamBodyAs::json_array(
            futures_util::stream::iter(&*blocks).map(block_as_json),
        )
    }
}

pub async fn blocks_with_cursor(
    State(s): State<AppState>,
    Query(range): Query<Range<usize>>,
) -> impl IntoResponse {
    let state = s.chain.read().await;
    let blocks = &state.blocks()[range] as *const [Block];

    // SAFETY: same as in `blocks`
    unsafe {
        axum_streams::StreamBodyAs::json_array(
            futures_util::stream::iter(&*blocks).map(block_as_json),
        )
    }
}

pub async fn validate_chain(State(s): State<AppState>) -> impl IntoResponse {
    // TODO: hashing pool/stream to avoid unecessary unsafe
    unsafe { Json(serde_json::json!({ "is_chain_valid": s.chain.write().await.validate_chain() })) }
}
