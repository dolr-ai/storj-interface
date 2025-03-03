use anyhow::Context;
use axum::{routing::get, Router};
use std::sync::Arc;
use tokio::{signal, sync::Notify};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new().route("/health", get(health));

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

async fn health() -> &'static str {
    "alive"
}
