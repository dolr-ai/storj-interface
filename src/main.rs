use anyhow::Context;
use axum::{
    routing::{get, post},
    Router,
};
use consts::ACCESS_GRANT;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::{signal, sync::Notify};

pub(crate) mod consts;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Lazy::force(&ACCESS_GRANT);

    let app = Router::new()
        .route("/health", get(health))
        .route("/duplicate", post(routes::duplicate::handler))
        .route("/move-to-nsfw", post(routes::move2nsfw::handler));

    let addr = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let server = axum::serve(addr, app);

    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    tokio::spawn(async move {
        if let Err(err) = signal::ctrl_c().await {
            eprintln!("Failed to listen for shutdown signal: {:#}", err);
        }
        notify_clone.notify_one();
    });

    println!("Starting to listen on http://localhost:3000");

    server
        .with_graceful_shutdown(async move {
            notify.notified().await;
            println!("Shutting down gracefully...");
        })
        .await
        .context("Server error")
}

/// Simple path to check that the server is running
async fn health() -> &'static str {
    "alive"
}
