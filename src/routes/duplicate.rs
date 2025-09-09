use axum::{extract::State, response::IntoResponse, Json};
use futures_util::StreamExt;
use reqwest::StatusCode;
use serde_json::json;
use std::collections::HashMap;
use std::process::Stdio;
use storj_interface::duplicate::Args;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::consts::{ACCESS_GRANT_NSFW, YRAL_NSFW_VIDEOS};
use crate::s3_client::S3Client;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Cloudflare returned non-ok status ({0}) when fetching the video")]
    Clouflare(StatusCode),

    #[error("S3 operation failed: {0}")]
    S3(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("err: {self}");
        let (status, message) = match self {
            Error::Network(_) | Error::Io(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error. Check server logs.",
            ),
            Error::Clouflare(StatusCode::NOT_FOUND) => (
                StatusCode::NOT_FOUND,
                "The video doesn't exist on cloudflare",
            ),
            Error::Clouflare(_) => (
                StatusCode::BAD_REQUEST,
                "The video couldn't fetched from cloudflare. Check server logs.",
            ),
            Error::S3(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "S3 storage operation failed. Check server logs.",
            ),
        };

        (
            status,
            Json(json!({
                "message": message
            })),
        )
            .into_response()
    }
}

pub async fn handler(
    State(s3_client): State<S3Client>,
    Json(Args {
        publisher_user_id,
        video_id,
        is_nsfw,
        metadata,
    }): Json<Args>,
) -> Result<impl IntoResponse, Error> {
    let source = format!(
        "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com/{video_id}/downloads/default.mp4",
    );

    let req = reqwest::get(source).await?;
    let status = req.status();

    if status != StatusCode::OK {
        return Err(Error::Clouflare(status));
    }

    if is_nsfw {
        // Use Storj for NSFW videos
        let bucket = YRAL_NSFW_VIDEOS.as_str();
        let grant = ACCESS_GRANT_NSFW.as_str();
        let dest = format!("sj://{bucket}/{publisher_user_id}/{video_id}.mp4");

        let mut stream = req.bytes_stream();
        let metadata_str = serde_json::to_string(&metadata)
            .expect("serialization to go through as we are guaranteed utf-8");

        let mut child = Command::new("uplink")
            .args([
                "cp",
                "--interactive=false",
                "--analytics=false",
                "--progress=false",
                format!("--metadata={metadata_str}").as_str(),
                "--access",
                grant,
                "-",
                dest.as_str(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()?;

        let mut pipe = child.stdin.take().expect("Stdin pipe to be opened for us");

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            pipe.write_all(&chunk).await?;
        }
    } else {
        // Use Hetzner S3 for SFW videos
        let key = format!("{publisher_user_id}/{video_id}.mp4");
        let stream = req.bytes_stream();

        // Convert metadata to HashMap for S3
        let mut s3_metadata = HashMap::new();
        for (k, v) in metadata.iter() {
            s3_metadata.insert(k.clone(), v.clone());
        }

        s3_client
            .upload_video_stream(&key, stream, &s3_metadata)
            .await
            .map_err(|e| Error::S3(e.to_string()))?;
    }

    Ok(())
}
