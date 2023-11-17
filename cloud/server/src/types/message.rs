use std::fmt::Debug;

use common_types::{DeviceId, DeviceResponse, LinkRequest, UserId, UserResponse};
use derive_more::From;
use tokio::sync::{mpsc, oneshot};

use crate::error::AppError;

#[derive(From, Debug)]
pub enum LinkMessage {
    UserLink(UserLink),
    NewUser(NewUser),
    NewDevice(NewDevice),
    #[from(ignore)]
    UserDropped(UserId),
    #[from(ignore)]
    DeviceDropped(DeviceId),
}

#[derive(Debug)]
pub struct UserLink {
    pub user_id: UserId,
    pub req: LinkRequest,
}

#[derive(Debug)]
pub struct NewUser {
    pub user_id: UserId,
    pub res_tx: oneshot::Sender<Result<mpsc::Receiver<UserResponse>, AppError>>,
}

#[derive(Debug)]
pub struct NewDevice {
    pub device_id: DeviceId,
    pub res_tx: oneshot::Sender<Result<mpsc::Receiver<DeviceResponse>, AppError>>,
}
