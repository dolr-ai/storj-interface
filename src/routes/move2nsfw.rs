use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::process::Stdio;
use tokio::process::Command;

use crate::consts::{ACCESS_GRANT, YRAL_NSFW_VIDEOS, YRAL_VIDEOS};

/// Args for moving a video to nsfw bucket
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
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("err: {self}");
        let (status, message) = match self {
            Error::Io(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error. Check server logs.",
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
    let source = format!(
        "sj://{YRAL_VIDEOS}/{}/{}.mp4",
        request.publisher_user_id, request.video_id
    );

    let dest = format!(
        "sj://{YRAL_NSFW_VIDEOS}/{}/{}.mp4",
        request.publisher_user_id, request.video_id
    );

    let mut child = Command::new("uplink")
        .args([
            "mv",
            "--interactive=false",
            "--analytics=false",
            "--progress=false",
            "--access",
            ACCESS_GRANT.as_str(),
            source.as_str(),
            dest.as_str(), // from stdin to dest
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .spawn()?;

    let status = child.wait().await?;

    if !status.success() {
        // TODO: analyze uplink's output to give a better error
        return Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "message": "uplink exitted with an error. Check server logs."
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "moved"
        })),
    ))
}
