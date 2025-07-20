use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use std::process::Stdio;
use storj_interface::move2nsfw::Args;
use tokio::process::Command;

use crate::consts::{ACCESS_GRANT_SFW, YRAL_NSFW_VIDEOS, YRAL_VIDEOS};

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
        "sj://{}/{}/{}.mp4",
        YRAL_VIDEOS.as_str(),
        request.publisher_user_id,
        request.video_id
    );

    let dest = format!(
        "sj://{}/{}/{}.mp4",
        YRAL_NSFW_VIDEOS.as_str(),
        request.publisher_user_id,
        request.video_id
    );

    let mut child = Command::new("uplink")
        .args([
            "mv",
            "--interactive=false",
            "--analytics=false",
            "--progress=false",
            "--access",
            ACCESS_GRANT_SFW.as_str(),
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
