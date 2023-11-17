use std::sync::Arc;

use axum::{
    extract::{Query, State, WebSocketUpgrade},
    response::Response,
};
use common_types::{DeviceId, UserId};
use serde::Deserialize;
use tokio::sync::oneshot;

use crate::{
    actors,
    error::{AppError, AppErrorExt},
    types::{
        message::{LinkMessage, NewDevice, NewUser},
        state::AppState,
    },
};

#[derive(Deserialize)]
pub struct ConnectRequest {
    pub id: String,
}

#[tracing::instrument(skip_all, err(Debug))]
pub async fn user_connect(
    State(state): State<Arc<AppState>>,
    Query(ConnectRequest { id }): Query<ConnectRequest>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let (res_tx, res_rx) = oneshot::channel();
    let id = UserId::from(id);
    let msg = LinkMessage::NewUser(NewUser {
        user_id: id.clone(),
        res_tx,
    });

    state.link_tx.send(msg).await.map_app_err()?;
    let user_rx = res_rx.await.map_app_err()??;

    Ok(ws.on_upgrade(|ws| async move {
        _ = actors::user::user_task(state, ws, id, user_rx).await;
    }))
}

#[tracing::instrument(skip_all, err(Debug))]
pub async fn device_connect(
    State(state): State<Arc<AppState>>,
    Query(ConnectRequest { id }): Query<ConnectRequest>,
    ws: WebSocketUpgrade,
) -> Result<Response, AppError> {
    let (res_tx, res_rx) = oneshot::channel();
    let id = DeviceId::from(id);
    let msg = LinkMessage::NewDevice(NewDevice {
        device_id: id.clone(),
        res_tx,
    });

    state.link_tx.send(msg).await.map_app_err()?;
    let device_tx = res_rx.await.map_app_err()??;

    Ok(ws.on_upgrade(|ws| async move {
        _ = actors::device::device_task(state, ws, id, device_tx).await;
    }))
}
