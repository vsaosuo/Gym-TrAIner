use derivative::Derivative;
use serde::{Deserialize, Serialize};

use crate::{id::DeviceId, UserId};

use super::workout::WorkoutType;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum LinkRequest {
    Connect { device_id: DeviceId },
    Disconnect,
}

// #[derive(Serialize, Deserialize, Derivative)]
// #[derivative(Debug)]
// #[serde(rename_all = "snake_case")]
// pub struct DeviceRequest {
//     pub session_id: SessionId,
//     pub req: VideoRequest,
// }

#[derive(Serialize, Deserialize, Derivative)]
#[derivative(Debug)]
#[serde(rename_all = "snake_case")]
pub enum VideoRequest {
    Start {
        user_id: UserId,
        workout_type: WorkoutType,
    },
    Frames(#[derivative(Debug = "ignore")] Vec<Frame>),
    Done,
    Cancel, // drop whatever video is currently being handled, if any
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Frame(#[serde(with = "serde_bytes")] pub Vec<u8>);
