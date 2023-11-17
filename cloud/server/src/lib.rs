mod actors;
mod constants;
mod error;
mod handlers;
mod types;

use std::sync::Arc;

use axum::{routing::get, Router};
use constants::CHANNEL_SIZE;
use firestore::FirestoreDb;
use google_cloud_default::WithAuthExt;
use google_cloud_storage::client::{Client, ClientConfig};
use tokio::sync::mpsc;
use tower::ServiceBuilder;
use tower_http::{
    cors::{self, CorsLayer},
    trace::TraceLayer,
};
use types::state::AppState;

pub async fn app() -> axum::Router {
    // cors layer
    let cors = CorsLayer::new().allow_origin(cors::Any);

    let (link_tx, link_rx) = mpsc::channel(CHANNEL_SIZE);

    tokio::spawn(actors::link::link_task(link_rx));

    let state = Arc::new(AppState {
        db: open_db().await,
        client: open_storage().await,
        link_tx,
    });

    Router::new()
        .route("/", get(|| async { "Hello, world!" }))
        .route("/user", get(handlers::connect::user_connect))
        .route("/device", get(handlers::connect::device_connect))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(cors),
        )
}

async fn open_db() -> FirestoreDb {
    FirestoreDb::new(std::env::var("PROJECT_ID").expect("PROJECT_ID is not set"))
        .await
        .expect("Failed to connect to database")
}

async fn open_storage() -> Client {
    match std::env::var("FIREBASE_STORAGE_EMULATOR_HOST") {
        Ok(host) => Client::new(
            ClientConfig {
                storage_endpoint: host,
                ..Default::default()
            }
            .with_auth()
            .await
            .expect("Failed to create ClientConfig"),
        ),
        Err(_) => Client::new(
            ClientConfig::default()
                .with_auth()
                .await
                .expect("Failed to create ClientConfig"),
        ),
    }
}
