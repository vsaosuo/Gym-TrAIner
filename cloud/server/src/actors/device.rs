mod video;

use crate::{
    actors::device::video::video_task,
    types::{message::LinkMessage, state::AppState},
};
use std::{sync::Arc, time::Duration};

use anyhow::{bail, Context};
use axum::extract::ws::{Message, WebSocket};
use common_types::{DeviceId, DeviceResponse, VideoRequest};
use tokio::{
    select,
    sync::mpsc::{self, UnboundedSender},
};

use self::video::VideoPart;

const NAME_WIDTH: usize = 4;

#[tracing::instrument(skip_all, err(Debug))]
pub async fn device_task(
    state: Arc<AppState>,
    ws: WebSocket,
    id: DeviceId,
    device_rx: mpsc::Receiver<DeviceResponse>,
) -> anyhow::Result<()> {
    _ = handle_device(&state, ws, id.clone(), device_rx).await;

    state.link_tx.send(LinkMessage::DeviceDropped(id)).await?;

    Ok(())
}

enum DeviceState {
    Disconnected,
    Connected,
}

#[derive(Debug, Default)]
enum VideoState {
    #[default]
    WaitStart,
    WaitDone {
        video_tx: UnboundedSender<VideoPart>,
    },
}

#[tracing::instrument(skip_all, err(Debug))]
async fn handle_device(
    app_state: &Arc<AppState>,
    mut ws: WebSocket,
    device_id: DeviceId,
    mut device_rx: mpsc::Receiver<DeviceResponse>,
) -> anyhow::Result<()> {
    const WS_TIMEOUT: Duration = Duration::from_secs(20);

    // NOTE: we can't use a separate task because we still need to respond to pings

    let mut device_state = DeviceState::Disconnected;
    let mut video_state = VideoState::WaitStart;

    loop {
        select! {
            msg = tokio::time::timeout(WS_TIMEOUT, ws.recv()) => {
                let Ok(msg) = msg else {
                    tracing::debug!("Connection with {:?} timed out", device_id);
                    break;
                };

                let Some(msg) = msg else {
                    tracing::debug!("{:?} disconnected", device_id);
                    break;
                };

                let msg = match msg? {
                    Message::Binary(msg) => {
                        tracing::debug!("Received device message from {:?}", device_id);
                        msg
                    }
                    Message::Ping(_) => {
                        // websocket automatically replies to pings
                        continue;
                    }
                    Message::Close(_) => {
                        tracing::debug!("{:?} closed the connection", device_id);
                        break;
                    }
                    msg => {
                        bail!("Unexpected message: {msg:?}")
                    }
                };

                handle_ws_msg(app_state, msg, &device_id, &mut video_state)?;
            }
            msg = device_rx.recv() => {
                let msg = msg.context("Link task stopped")?;
                handle_device_msg(msg, &device_id, &mut ws, &mut device_state).await?;
            }
        }
    }

    Ok(())
}

#[tracing::instrument(skip_all, err(Debug))]
async fn handle_device_msg(
    msg: DeviceResponse,
    device_id: &DeviceId,
    ws: &mut WebSocket,
    state: &mut DeviceState,
) -> anyhow::Result<()> {
    match (state, &msg) {
        (state @ DeviceState::Disconnected, DeviceResponse::Connected { user_id }) => {
            tracing::debug!("{:?} connected to {:?}", device_id, user_id);
            *state = DeviceState::Connected;
        }
        (state @ DeviceState::Connected { .. }, DeviceResponse::Disconnected) => {
            tracing::debug!("{:?} disconnected", device_id);
            *state = DeviceState::Disconnected;
        }
        (_, msg) => {
            bail!("Unexpected device response: {:?}", msg)
        }
    };

    let res = serde_json::to_string(&msg)?;
    ws.send(Message::Text(res)).await?;

    Ok(())
}

fn handle_ws_msg(
    app_state: &Arc<AppState>,
    msg: Vec<u8>,
    device_id: &DeviceId,
    state: &mut VideoState,
) -> anyhow::Result<()> {
    let req: VideoRequest = bincode::deserialize(&msg)?;
    tracing::debug!("Received video request from {:?}: {:?}", device_id, req);

    match (std::mem::take(state), req) {
        (
            VideoState::WaitStart,
            VideoRequest::Start {
                user_id,
                workout_type,
            },
        ) => {
            let (video_tx, video_rx) = mpsc::unbounded_channel();
            tokio::spawn(video_task(
                app_state.clone(),
                video_rx,
                user_id,
                workout_type,
            ));
            *state = VideoState::WaitDone { video_tx };
        }
        (VideoState::WaitDone { video_tx }, VideoRequest::Frames(frames)) => {
            video_tx.send(VideoPart::Frames(frames))?;
            *state = VideoState::WaitDone { video_tx };
        }
        (VideoState::WaitDone { video_tx }, VideoRequest::Done) => {
            video_tx.send(VideoPart::Done)?;
            // video_tx gets dropped, but video is done so it will get processed
        }
        (_, VideoRequest::Cancel) => {
            // state becomes WaitStart, video_tx gets dropped if it exists
        }
        (state, req) => {
            bail!("Invalid: (state, req) = ({state:?}, {req:?})")
        }
    }

    Ok(())
}
