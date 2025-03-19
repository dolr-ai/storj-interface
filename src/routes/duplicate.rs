use axum::{response::IntoResponse, Json};
use futures_util::StreamExt;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::BTreeMap, process::Stdio};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::consts::{ACCESS_GRANT, YRAL_NSFW_VIDEOS, YRAL_VIDEOS};

/// Args for duplication request
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Args {
    /// The publisher user principal supplied to off chain agent
    ///
    /// This used as directory key
    publisher_user_id: String,
    /// The video id on cloudflare
    ///
    /// This is used as object key
    video_id: String,
    /// Whether the video contains nsfw content
    ///
    /// This is used for segregation
    is_nsfw: bool,
    /// key-value pair to be added to video's metadata on storj
    metadata: BTreeMap<String, String>,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("Cloudflare returned non-ok status ({0}) when fetching the video")]
    Clouflare(StatusCode),
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

pub async fn handler(Json(request): Json<Args>) -> Result<impl IntoResponse, Error> {
    let bucket = if request.is_nsfw {
        YRAL_NSFW_VIDEOS
    } else {
        YRAL_VIDEOS
    };

    let dest = format!(
        "sj://{bucket}/{}/{}.mp4",
        request.publisher_user_id, request.video_id
    );

    let source = format!(
        "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com/{}/downloads/default.mp4",
        request.video_id
    );

    let req = reqwest::get(source).await?;

    let status = req.status();

    if status != StatusCode::OK {
        return Err(Error::Clouflare(status));
    }

    let mut stream = req.bytes_stream();

    let mut child = Command::new("uplink")
        .args([
            "cp",
            "--interactive=false",
            "--analytics=false",
            "--progress=false",
            "--access",
            ACCESS_GRANT.as_str(),
            "-",
            dest.as_str(), // from stdin to dest
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;

    let mut pipe = child.stdin.take().expect("Stdin pipe to be opened for us");

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        pipe.write_all(&chunk).await?;
    }

    Ok(())
}
