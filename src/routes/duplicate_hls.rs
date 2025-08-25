use axum::{body::Body, extract::Query, response::IntoResponse, Json};
use futures_util::StreamExt;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::consts::{ACCESS_GRANT_NSFW, ACCESS_GRANT_SFW, YRAL_NSFW_VIDEOS, YRAL_VIDEOS};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Hyper(#[from] axum::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("err: {self}");
        let (status, message) = match self {
            Error::Network(_) | Error::Io(_) | Error::Hyper(_) => (
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

#[derive(Deserialize)]
pub struct HlsUploadParams {
    video_id: String,
    is_nsfw: bool,
    hls_file_name: String,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

pub async fn handler(
    Query(params): Query<HlsUploadParams>,
    body: Body,
) -> Result<impl IntoResponse, Error> {
    let (bucket, grant) = if params.is_nsfw {
        (YRAL_NSFW_VIDEOS.as_str(), ACCESS_GRANT_NSFW.as_str())
    } else {
        (YRAL_VIDEOS.as_str(), ACCESS_GRANT_SFW.as_str())
    };

    let dest = format!(
        "sj://{bucket}/{}/hls/{}",
        params.video_id, params.hls_file_name
    );

    let metadata_str = serde_json::to_string(&params.metadata)
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
            dest.as_str(), // from stdin to dest
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let mut pipe = child.stdin.take().expect("Stdin pipe to be opened for us");

    // Stream the body data to uplink
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(data) => {
                pipe.write_all(&data).await?;
            }
            Err(e) => {
                return Err(Error::Hyper(e));
            }
        }
    }

    pipe.flush().await?;
    drop(pipe); // Close stdin to signal EOF

    // Note: We don't wait for the uplink process to complete
    // This matches the behavior of the /duplicate endpoint

    Ok(())
}
