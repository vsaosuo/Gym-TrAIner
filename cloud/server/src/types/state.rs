use firestore::FirestoreDb;
use google_cloud_storage::client::Client as StorageClient;
use tokio::sync::mpsc;

use super::message::LinkMessage;

/// Shared state used by all routes.
pub struct AppState {
    pub db: FirestoreDb,
    pub client: StorageClient,
    pub link_tx: mpsc::Sender<LinkMessage>,
}
