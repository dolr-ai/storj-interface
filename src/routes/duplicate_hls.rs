use axum::{
    body::Body,
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::process::Stdio;
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

    #[error(transparent)]
    Hyper(#[from] axum::Error),

    #[error("S3 operation failed: {0}")]
    S3(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("err: {self}");
        let (status, message) = match self {
            Error::Network(_) | Error::Io(_) | Error::Hyper(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error. Check server logs.",
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

#[derive(Deserialize)]
pub struct HlsUploadParams {
    video_id: String,
    is_nsfw: bool,
    hls_file_name: String,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

pub async fn handler(
    State(s3_client): State<S3Client>,
    Query(params): Query<HlsUploadParams>,
    body: Body,
) -> Result<impl IntoResponse, Error> {
    // Collect the body data
    let mut stream = body.into_data_stream();
    let mut body_data = Vec::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(data) => {
                body_data.extend_from_slice(&data);
            }
            Err(e) => {
                return Err(Error::Hyper(e));
            }
        }
    }

    if params.is_nsfw {
        // Use Storj for NSFW videos
        let bucket = YRAL_NSFW_VIDEOS.as_str();
        let grant = ACCESS_GRANT_NSFW.as_str();
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
                dest.as_str(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let mut pipe = child.stdin.take().expect("Stdin pipe to be opened for us");
        pipe.write_all(&body_data).await?;
        pipe.flush().await?;
        drop(pipe); // Close stdin to signal EOF
    } else {
        // Use Hetzner S3 for SFW videos
        let key = format!("{}/hls/{}", params.video_id, params.hls_file_name);

        // Convert metadata to HashMap for S3
        let mut s3_metadata = HashMap::new();
        for (k, v) in params.metadata.iter() {
            s3_metadata.insert(k.clone(), v.clone());
        }

        s3_client
            .upload_hls_segment(&key, Bytes::from(body_data), &s3_metadata)
            .await
            .map_err(|e| Error::S3(e.to_string()))?;
    }

    Ok(())
}
