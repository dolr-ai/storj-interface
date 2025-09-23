use axum::{
    body::Body,
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use bytes::Bytes;
use http_body_util::BodyExt;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::consts::{ACCESS_GRANT_NSFW, ACCESS_GRANT_SFW, YRAL_NSFW_VIDEOS, YRAL_VIDEOS};
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

async fn upload_hls_to_storj(
    video_id: &str,
    hls_file_name: &str,
    metadata: &BTreeMap<String, String>,
    body_data: &[u8],
    is_nsfw: bool,
) -> Result<(), Error> {
    let (bucket, grant) = if is_nsfw {
        (YRAL_NSFW_VIDEOS.as_str(), ACCESS_GRANT_NSFW.as_str())
    } else {
        (YRAL_VIDEOS.as_str(), ACCESS_GRANT_SFW.as_str())
    };
    let dest = format!("sj://{bucket}/{video_id}/hls/{hls_file_name}");

    let metadata_str = serde_json::to_string(metadata)
        .expect("serialization to go through as we are guaranteed utf-8");

    let mut child = Command::new("uplink")
        .args([
            "cp",
            "--interactive=false",
            "--analytics=false",
            "--progress=false",
            "--immutable=false",  // Allow overwriting existing files
            format!("--metadata={metadata_str}").as_str(),
            "--access",
            grant,
            "-",
            dest.as_str(),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())  // Capture stderr for better error messages
        .spawn()?;

    let mut pipe = child.stdin.take().expect("Stdin pipe to be opened for us");
    pipe.write_all(body_data).await?;
    pipe.flush().await?;
    drop(pipe); // Close stdin to signal EOF

    let output = child.wait_with_output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Storj HLS upload error for {video_id}/{hls_file_name}: {stderr}");
        return Err(Error::Io(std::io::Error::other(format!(
            "uplink command failed: {stderr}"
        ))));
    }

    Ok(())
}

async fn upload_hls_to_s3(
    s3_client: &S3Client,
    video_id: &str,
    hls_file_name: &str,
    metadata: &BTreeMap<String, String>,
    body_data: &Bytes,
) -> Result<(), Error> {
    let key = format!("{video_id}/hls/{hls_file_name}");

    // Convert metadata to HashMap for S3
    let mut s3_metadata = HashMap::new();
    for (k, v) in metadata.iter() {
        s3_metadata.insert(k.clone(), v.clone());
    }

    s3_client
        .upload_hls_segment(&key, body_data.clone(), &s3_metadata)
        .await
        .map_err(|e| {
            eprintln!("S3 HLS upload error for {video_id}/{hls_file_name}: {e}",);
            Error::S3(e.to_string())
        })?;

    Ok(())
}

pub async fn handler(
    State(s3_client): State<S3Client>,
    Query(params): Query<HlsUploadParams>,
    body: Body,
) -> Result<impl IntoResponse, Error> {
    // Use the cleaner collection method
    let body_data = body.collect().await.map_err(Error::Hyper)?.to_bytes();

    let params = Arc::new(params);
    let body_data = Arc::new(body_data);

    let mut join_set = tokio::task::JoinSet::new();

    // Always upload to Storj
    {
        let params = params.clone();
        let body_data = body_data.clone();
        join_set.spawn(async move {
            upload_hls_to_storj(
                &params.video_id,
                &params.hls_file_name,
                &params.metadata,
                &body_data.to_vec(),
                params.is_nsfw,
            )
            .await
        });
    }

    // Additionally upload to S3 for SFW videos
    if !params.is_nsfw {
        join_set.spawn(async move {
            upload_hls_to_s3(
                &s3_client,
                &params.video_id,
                &params.hls_file_name,
                &params.metadata,
                &body_data,
            )
            .await
        });
    }

    while let Some(result) = join_set.join_next().await {
        result.map_err(|e| Error::Io(std::io::Error::other(e)))??;
    }

    Ok(())
}
