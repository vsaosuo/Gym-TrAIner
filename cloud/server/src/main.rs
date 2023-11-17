use std::net::SocketAddr;

use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, EnvFilter,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // for logging
    tracing_subscriber::registry()
        .with(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "server=debug,tower_http=trace".into()),
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .with_file(true)
                .with_line_number(true),
        )
        .init();

    if let Ok(dir) = std::env::current_dir() {
        tracing::debug!("Working directory is: {dir:?}");
    }

    // bind on either IPv4 or IPv6
    let addr = SocketAddr::from(([0; 8], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(server::app().await.into_make_service())
        .await
        .expect("failed to start server");

    Ok(())
}
