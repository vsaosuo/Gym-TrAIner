use std::sync::Arc;

use anyhow::{bail, Context};
use axum::extract::ws::{Message, WebSocket};
use common_types::{LinkRequest, UserId, UserResponse};
use tokio::{select, sync::mpsc};

use crate::types::{
    message::{LinkMessage, UserLink},
    state::AppState,
};

#[derive(Clone, Copy, Debug)]
enum UserState {
    Disconnected,
    PendingConnect,
    Connected,
    PendingDisconnect,
}

#[tracing::instrument(skip_all, err(Debug))]
pub async fn user_task(
    state: Arc<AppState>,
    ws: WebSocket,
    user_id: UserId,
    user_rx: mpsc::Receiver<UserResponse>,
) -> anyhow::Result<()> {
    // do nothing with the result, since it will be logged anyway
    let link_tx = &state.link_tx;
    _ = handle_user(ws, user_id.clone(), link_tx, user_rx).await;

    // at the end of handling, signal for the user to be dropped
    link_tx.send(LinkMessage::UserDropped(user_id)).await?;

    Ok(())
}

async fn handle_user(
    mut ws: WebSocket,
    user_id: UserId,
    link_tx: &mpsc::Sender<LinkMessage>,
    mut user_rx: mpsc::Receiver<UserResponse>,
) -> anyhow::Result<()> {
    let mut state = UserState::Disconnected;

    loop {
        select! {
            msg = ws.recv() => {
                let Some(msg) = msg else {
                    tracing::debug!("{:?} dropped the connection", user_id);
                    break;
                };

                let msg = match msg? {
                    Message::Text(msg) => msg,
                    Message::Ping(_) => {
                        // websocket automatically replies to pings
                        continue;
                    }
                    Message::Close(_) => {
                        tracing::debug!("{:?} closed the connection", user_id);
                        break;
                    }
                    msg => {
                        bail!("Unexpected message: {msg:?}")
                    }
                };

                handle_ws_msg(msg, &user_id, &mut state, link_tx).await?;
            }
            // TODO: this should have a timeout in the future...
            msg = user_rx.recv() => {
                let msg = msg.context("Link task stopped")?;
                handle_user_msg(msg, &user_id, &mut ws, &mut state).await?;
            }
        }
    }

    Ok(())
}

async fn handle_ws_msg(
    msg: String,
    user_id: &UserId,
    state: &mut UserState,
    link_tx: &mpsc::Sender<LinkMessage>,
) -> anyhow::Result<()> {
    let req: LinkRequest = serde_json::from_str(&msg)?;
    tracing::debug!("Received link request from {:?}: {:?}", user_id, req);

    match (*state, &req) {
        (UserState::Disconnected, LinkRequest::Connect { device_id }) => {
            tracing::debug!("{:?} requested connection to {:?}", user_id, device_id);
            link_tx
                .send(
                    UserLink {
                        user_id: user_id.clone(),
                        req,
                    }
                    .into(),
                )
                .await?;
            *state = UserState::PendingConnect;
        }
        (UserState::Connected, LinkRequest::Disconnect) => {
            tracing::debug!("{:?} requested disconnect", user_id);
            link_tx
                .send(
                    UserLink {
                        user_id: user_id.clone(),
                        req,
                    }
                    .into(),
                )
                .await?;
            *state = UserState::PendingDisconnect;
        }
        // this has the potential of going wrong
        // let's log it for now and see what happens...
        (state, req) => {
            // wrong state
            // this shouldn't happen if we handled disconnect + drop race condition correctly
            tracing::warn!("Invalid: (state, req) = ({state:?}, {req:?}), message ignored");
        }
    };

    Ok(())
}

async fn handle_user_msg(
    msg: UserResponse,
    user_id: &UserId,
    ws: &mut WebSocket,
    state: &mut UserState,
) -> anyhow::Result<()> {
    match (*state, &msg) {
        (UserState::PendingConnect, UserResponse::Connected { device_id }) => {
            tracing::debug!("{:?} connected to {:?}", user_id, device_id);
            *state = UserState::Connected;
        }
        (UserState::PendingDisconnect, UserResponse::Disconnected) => {
            tracing::debug!("{:?} disconnected", user_id);
            *state = UserState::Disconnected;
        }
        (UserState::PendingConnect, UserResponse::NoSuchDevice) => {
            tracing::debug!("{:?} tried to connect to non-existent device", user_id);
            *state = UserState::Disconnected;
        }
        (_, UserResponse::Dropped) => {
            tracing::debug!("Connection for {:?} was dropped", user_id);
            *state = UserState::Disconnected;
        }
        // this can be kept since it represents an internal error
        _ => bail!("Unexpected user response: {:?}", msg),
    };

    // forward
    let res = serde_json::to_string(&msg)?;
    ws.send(Message::Text(res)).await?;

    Ok(())
}
