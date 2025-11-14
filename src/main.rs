use anyhow::Context;
use axum::{
    extract::{DefaultBodyLimit, Request},
    http::HeaderMap,
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use consts::{
    ACCESS_GRANT_NSFW, ACCESS_GRANT_SFW, HETZNER_S3_ACCESS_KEY, HETZNER_S3_BUCKET,
    HETZNER_S3_ENDPOINT, HETZNER_S3_REGION, HETZNER_S3_SECRET_KEY, SERVICE_SECRET_TOKEN,
    YRAL_VIDEOS,
};
use once_cell::sync::Lazy;
use reqwest::{header::AUTHORIZATION, StatusCode};
use std::sync::Arc;
use tokio::{signal, sync::Notify};

pub(crate) mod consts;
mod routes;
mod s3_client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Force loading of Storj configuration
    Lazy::force(&ACCESS_GRANT_SFW);
    Lazy::force(&ACCESS_GRANT_NSFW);
    Lazy::force(&YRAL_VIDEOS);
    Lazy::force(&SERVICE_SECRET_TOKEN);

    // Force loading of Hetzner S3 configuration
    Lazy::force(&HETZNER_S3_ENDPOINT);
    Lazy::force(&HETZNER_S3_BUCKET);
    Lazy::force(&HETZNER_S3_ACCESS_KEY);
    Lazy::force(&HETZNER_S3_SECRET_KEY);
    Lazy::force(&HETZNER_S3_REGION);

    // Initialize S3 client
    let s3_client = s3_client::S3Client::new().await;

    let app = Router::new()
        .route(
            "/duplicate",
            post(routes::duplicate::handler)
                .with_state(s3_client.clone())
                .layer(middleware::from_fn(authorize)),
        )
        .route(
            "/duplicate_raw/upload",
            post(routes::duplicate::handler_raw_upload_initial)
                .with_state(s3_client.clone())
                .layer(DefaultBodyLimit::max(500 * 1024 * 1024)), // 500MB limit for raw video upload
        )
        .route(
            "/duplicate_raw/finalize",
            post(routes::duplicate::handler_raw_finalize).with_state(s3_client.clone()),
        )
        // NOTE: This will be removed as the upload happens in the very end of the pipeline and nsfw flag is passed into duplicate
        .route(
            "/move-to-nsfw",
            post(routes::move2nsfw::handler)
                .with_state(s3_client.clone())
                .layer(middleware::from_fn(authorize)),
        )
        .route(
            "/hls/duplicate",
            post(routes::duplicate_hls::handler)
                .with_state(s3_client.clone())
                .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB limit for HLS files
                .layer(middleware::from_fn(authorize)),
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
