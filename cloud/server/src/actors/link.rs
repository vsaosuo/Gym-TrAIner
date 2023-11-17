use std::{
    collections::{hash_map::Entry, HashMap},
    fmt::Debug,
};

use common_types::{DeviceId, DeviceResponse, LinkRequest, UserId, UserResponse};
use thiserror::Error;
use tokio::sync::mpsc;

use crate::{
    constants::CHANNEL_SIZE,
    error::AppError,
    types::message::{LinkMessage, NewDevice, NewUser, UserLink},
};

enum UserConnection {
    Connected(DeviceId),
    Disconnected,
    Dropped,
}

enum DeviceConnection {
    Connected(UserId),
    Disconnected,
}

struct UserEntry {
    connection: UserConnection,
    res_tx: mpsc::Sender<UserResponse>,
}

struct DeviceEntry {
    connection: DeviceConnection,
    res_tx: mpsc::Sender<DeviceResponse>,
}

#[derive(Default)]
struct LinkManager {
    users: HashMap<UserId, UserEntry>,
    devices: HashMap<DeviceId, DeviceEntry>,
}

macro_rules! log_if_err {
    ($res:expr) => {
        if let Err(e) = $res {
            tracing::info!("{}", e);
        }
    };
    ($fmt:literal, $res:expr) => {
        if let Err(e) = $res {
            tracing::info!($fmt, e);
        }
    };
}

#[derive(Debug, Error)]
pub enum LinkError {
    #[error("No more link messages")]
    NoMoreMessages,
    #[error("User entry should exist but doesn't")]
    NoUserEntry,
    #[error("Device entry should exist but doesn't")]
    NoDeviceEntry,
    #[error("User should not have matching device")]
    HasDevice,
    #[error("Device should not have matching user")]
    HasUser,
    #[error("User is already disconnected but shouldn't be")]
    DisconnectedUser,
    #[error("Expected matching connected user ID")]
    NoMatchingUser,
    #[error("Expected matching connected device ID")]
    NoMatchingDevice,
}

type LinkResult<T> = Result<T, LinkError>;

#[tracing::instrument(skip_all, err(Debug))]
pub async fn link_task(mut msg_rx: mpsc::Receiver<LinkMessage>) -> LinkResult<()> {
    let mut lm = LinkManager::default();

    loop {
        let msg = msg_rx.recv().await.ok_or(LinkError::NoMoreMessages)?;

        lm.handle_message(msg).await?;
    }
}

impl LinkManager {
    async fn handle_message(&mut self, msg: LinkMessage) -> LinkResult<()> {
        match msg {
            LinkMessage::UserLink(user_link) => self.handle_user_link(user_link).await?,
            LinkMessage::NewUser(new_user) => self.handle_new_user(new_user).await,
            LinkMessage::NewDevice(new_device) => self.handle_new_device(new_device).await,
            LinkMessage::UserDropped(user_id) => self.handle_user_dropped(user_id).await?,
            LinkMessage::DeviceDropped(device_id) => self.handle_device_dropped(device_id).await?,
        }

        Ok(())
    }

    async fn handle_user_link(&mut self, UserLink { user_id, req }: UserLink) -> LinkResult<()> {
        // entry should definitely exist
        let user_entry = self.users.get_mut(&user_id).ok_or(LinkError::NoUserEntry)?;

        match req {
            LinkRequest::Connect { device_id } => {
                tracing::debug!("{user_id:?} requested to connect to {device_id:?}");

                // entry might not exist if the device is not connected
                let Some(device_entry) = self
                    .devices
                    .get_mut(&device_id) else {
                        log_if_err!(user_entry.res_tx.send(UserResponse::NoSuchDevice).await);
                        return Ok(());
                    };

                if let UserConnection::Connected(..) = user_entry.connection {
                    return Err(LinkError::HasDevice);
                }
                if let DeviceConnection::Connected(..) = device_entry.connection {
                    return Err(LinkError::HasUser);
                }

                user_entry.connection = UserConnection::Connected(device_id.clone());
                log_if_err!(
                    user_entry
                        .res_tx
                        .send(UserResponse::Connected { device_id })
                        .await
                );

                device_entry.connection = DeviceConnection::Connected(user_id.clone());

                log_if_err!(
                    device_entry
                        .res_tx
                        .send(DeviceResponse::Connected { user_id })
                        .await
                );
            }
            LinkRequest::Disconnect => {
                tracing::debug!("{user_id:?} requested to disconnect");

                let device_id = match &user_entry.connection {
                    UserConnection::Connected(device_id) => device_id,
                    UserConnection::Dropped => {
                        // drop already happened, nothing to do here
                        user_entry.connection = UserConnection::Disconnected;
                        return Ok(());
                    }
                    _ => return Err(LinkError::DisconnectedUser),
                };

                // connected case happens here
                let device_entry = self
                    .devices
                    .get_mut(device_id)
                    .ok_or(LinkError::NoDeviceEntry)?;

                match &device_entry.connection {
                    DeviceConnection::Connected(id) if *id == user_id => (),
                    _ => return Err(LinkError::NoMatchingUser),
                }

                user_entry.connection = UserConnection::Disconnected;
                log_if_err!(user_entry.res_tx.send(UserResponse::Disconnected).await);

                device_entry.connection = DeviceConnection::Disconnected;
                log_if_err!(device_entry.res_tx.send(DeviceResponse::Disconnected).await);
            }
        }

        Ok(())
    }

    async fn handle_new_user(&mut self, NewUser { user_id, res_tx }: NewUser) {
        tracing::debug!("{user_id:?} connected");

        // check for duplicate
        match self.users.entry(user_id) {
            Entry::Occupied(_) => log_if_err!(
                "Failed to send: {:?}",
                res_tx.send(Err(AppError::DuplicateId))
            ),
            Entry::Vacant(v) => {
                let (user_tx, user_rx) = mpsc::channel(CHANNEL_SIZE);
                v.insert(UserEntry {
                    connection: UserConnection::Disconnected,
                    res_tx: user_tx,
                });
                log_if_err!("Failed to send: {:?}", res_tx.send(Ok(user_rx)));
            }
        }
    }

    async fn handle_new_device(&mut self, NewDevice { device_id, res_tx }: NewDevice) {
        tracing::debug!("{device_id:?} connected");

        // check for duplicate
        match self.devices.entry(device_id) {
            Entry::Occupied(_) => log_if_err!(
                "Failed to send: {:?}",
                res_tx.send(Err(AppError::DuplicateId))
            ),
            Entry::Vacant(v) => {
                let (device_tx, device_rx) = mpsc::channel(CHANNEL_SIZE);
                v.insert(DeviceEntry {
                    connection: DeviceConnection::Disconnected,
                    res_tx: device_tx,
                });
                log_if_err!("Failed to send: {:?}", res_tx.send(Ok(device_rx)));
            }
        }
    }

    async fn handle_user_dropped(&mut self, user_id: UserId) -> LinkResult<()> {
        tracing::debug!("{user_id:?} dropped the connection");

        let Entry::Occupied(mut to_remove) = self.users.entry(user_id.clone()) else {
            return Err(LinkError::NoUserEntry);
        };
        let user_entry = to_remove.get_mut();

        // signal the disconnect if needed
        if let UserConnection::Connected(device_id) = &user_entry.connection {
            let device_entry = self
                .devices
                .get_mut(device_id)
                .ok_or(LinkError::NoDeviceEntry)?;

            match &device_entry.connection {
                DeviceConnection::Connected(id) if *id == user_id => (),
                _ => return Err(LinkError::NoMatchingUser),
            }

            device_entry.connection = DeviceConnection::Disconnected;
            log_if_err!(device_entry.res_tx.send(DeviceResponse::Disconnected).await);
        }

        // remove entry
        to_remove.remove();

        Ok(())
    }

    async fn handle_device_dropped(&mut self, device_id: DeviceId) -> LinkResult<()> {
        tracing::debug!("{device_id:?} dropped the connection");

        let Entry::Occupied(mut to_remove) = self.devices.entry(device_id.clone()) else {
            return Err(LinkError::NoDeviceEntry);
        };
        let device_entry = to_remove.get_mut();

        // signal the disconnect if needed
        if let DeviceConnection::Connected(user_id) = &device_entry.connection {
            let user_entry = self.users.get_mut(user_id).ok_or(LinkError::NoUserEntry)?;

            match &user_entry.connection {
                UserConnection::Connected(id) if *id == device_id => (),
                _ => return Err(LinkError::NoMatchingDevice),
            };

            user_entry.connection = UserConnection::Dropped;
            log_if_err!(user_entry.res_tx.send(UserResponse::Dropped).await);
        }

        // remove entry
        to_remove.remove();

        Ok(())
    }
}
