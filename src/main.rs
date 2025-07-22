use anyhow::Context;
use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use consts::{ACCESS_GRANT_NSFW, ACCESS_GRANT_SFW, SERVICE_SECRET_TOKEN};
use once_cell::sync::Lazy;
use reqwest::{header::AUTHORIZATION, StatusCode};
use std::sync::Arc;
use tokio::{signal, sync::Notify};

pub(crate) mod consts;
mod routes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Lazy::force(&ACCESS_GRANT_SFW);
    Lazy::force(&ACCESS_GRANT_NSFW);
    Lazy::force(&SERVICE_SECRET_TOKEN);

    let app = Router::new()
        .route(
            "/duplicate",
            post(routes::duplicate::handler).layer(middleware::from_fn(authorize)),
        )
        .route(
            "/move-to-nsfw",
            post(routes::move2nsfw::handler).layer(middleware::from_fn(authorize)),
        )
        .route("/health", get(health));

    let addr = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    let server = axum::serve(addr, app);

    let notify = Arc::new(Notify::new());
    let notify_clone = notify.clone();

    tokio::spawn(async move {
        if let Err(err) = signal::ctrl_c().await {
            eprintln!("Failed to listen for shutdown signal: {err:#}");
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

/// A dead simple authorization check based on a shared secret
async fn authorize(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    let auth = headers.get(AUTHORIZATION).ok_or(StatusCode::UNAUTHORIZED)?;
    let auth = auth.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;

    if auth != SERVICE_SECRET_TOKEN.as_str() {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}
