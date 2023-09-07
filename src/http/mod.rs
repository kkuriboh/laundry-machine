use std::{collections::HashMap, net::SocketAddr};

use axum::{
    extract::ws::Message,
    routing::{get, post},
    Router,
};
use tokio::sync::{Mutex, RwLock};

use crate::primitives::{block::Block, Chain};

use handlers::*;
use ws_handlers::*;

mod data_helpers;
mod handlers;
mod ws_handlers;

#[derive(Clone)]
pub(crate) enum SocketConnectionType {
    Income,
    Outcome,
}

struct _AppState {
    chain: RwLock<Chain>,
    sockets: tokio::sync::broadcast::Sender<Message>,
}

impl _AppState {
    fn make() -> AppState {
        let (tx, _) = tokio::sync::broadcast::channel::<Message>(1);
        let chain = Chain::new(Block::GENESIS).into();

        Box::leak(Box::new(Self { chain, sockets: tx }))
    }
}

struct _WSState {
    app_state: AppState,
    connections: Mutex<HashMap<SocketAddr, SocketConnectionType>>,
}

impl _WSState {
    fn make(app_state: AppState) -> WSState {
        Box::leak(Box::new(Self {
            app_state,
            connections: Mutex::new(HashMap::new()),
        }))
    }
}

type AppState = &'static _AppState;
type WSState = &'static _WSState;

fn make_state() -> (AppState, WSState) {
    let app_state = _AppState::make();
    let ws_state = _WSState::make(app_state);
    (app_state, ws_state)
}

pub fn make_router() -> Router {
    let (app_state, ws_state) = make_state();

    let websocket_router = Router::new()
        .route("/ws", get(ws_index))
        .route("/peers", get(peers))
        .route("/peers/add", post(add_peer))
        .with_state(ws_state);

    Router::new()
        .route("/", get(index))
        .route("/blocks", get(blocks))
        .route("/blocks/with_cursor", get(blocks_with_cursor))
        .route("/mine_block", post(mine_block))
        .route("/validate_chain", get(validate_chain))
        .with_state(app_state)
        .merge(websocket_router)
}
