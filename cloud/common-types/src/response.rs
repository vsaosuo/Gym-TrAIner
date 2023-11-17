use serde::{Deserialize, Serialize};

use super::id::{DeviceId, UserId};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "status")]
pub enum UserResponse {
    Connected { device_id: DeviceId },
    Disconnected,
    NoSuchDevice,
    Dropped,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DeviceResponse {
    Connected { user_id: UserId },
    Disconnected,
}
