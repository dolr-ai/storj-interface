use axum::{body::Body, extract::State, response::IntoResponse, Json};
use futures_util::StreamExt;
use http_body_util::BodyExt;
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::process::Stdio;
use storj_interface::duplicate::Args;
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

    #[error("Cloudflare returned non-ok status ({0}) when fetching the video")]
    Clouflare(StatusCode),

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

async fn upload_to_storj(
    publisher_user_id: &str,
    video_id: &str,
    metadata: &BTreeMap<String, String>,
    mut stream: impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
    is_nsfw: bool,
) -> Result<(), Error> {
    let (bucket, grant) = if is_nsfw {
        (YRAL_NSFW_VIDEOS.as_str(), ACCESS_GRANT_NSFW.as_str())
    } else {
        (YRAL_VIDEOS.as_str(), ACCESS_GRANT_SFW.as_str())
    };
    let dest = format!("sj://{bucket}/{publisher_user_id}/{video_id}.mp4");

    let metadata_str = serde_json::to_string(metadata)
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

    drop(pipe);
    let status = child.wait().await?;
    if !status.success() {
        return Err(Error::Io(std::io::Error::other(format!(
            "uplink command failed with status: {status}"
        ))));
    }

    Ok(())
}

async fn upload_to_s3(
    s3_client: &S3Client,
    publisher_user_id: &str,
    video_id: &str,
    metadata: &BTreeMap<String, String>,
    stream: impl futures_util::Stream<Item = Result<bytes::Bytes, reqwest::Error>>,
) -> Result<(), Error> {
    let key = format!("{publisher_user_id}/{video_id}.mp4");

    // Convert metadata to HashMap for S3
    let mut s3_metadata = HashMap::new();
    for (k, v) in metadata.iter() {
        s3_metadata.insert(k.clone(), v.clone());
    }

    s3_client
        .upload_video_stream(&key, stream, &s3_metadata)
        .await
        .map_err(|e| {
            eprintln!("S3 upload error for {publisher_user_id}/{video_id}: {e:?}",);
            Error::S3(format!("{e:?}"))
        })?;

    Ok(())
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

    if !is_nsfw {
        // Collect all bytes into memory for SFW videos as we need to upload to both Storj and S3
        // Stream clone aint working
        let body = req.bytes().await?;
        let body_clone = body.clone();

        // Create streams from the collected bytes
        let storj_stream = futures_util::stream::once(async move { Ok(body) });
        let s3_stream = futures_util::stream::once(async move { Ok(body_clone) });

        let storj_upload = upload_to_storj(
            &publisher_user_id,
            &video_id,
            &metadata,
            Box::pin(storj_stream),
            is_nsfw,
        );
        let s3_upload = upload_to_s3(
            &s3_client,
            &publisher_user_id,
            &video_id,
            &metadata,
            s3_stream,
        );

        tokio::try_join!(storj_upload, s3_upload)?;
    } else {
        upload_to_storj(
            &publisher_user_id,
            &video_id,
            &metadata,
            req.bytes_stream(),
            is_nsfw,
        )
        .await?;
    }

    Ok(())
}

#[derive(Deserialize)]
pub struct RawUploadInitialParams {
    publisher_user_id: String,
    video_id: String,
    is_nsfw: bool,
    #[serde(default)]
    ttl_hours: Option<u32>,
}

#[derive(Deserialize)]
pub struct RawFinalizeParams {
    publisher_user_id: String,
    video_id: String,
    is_nsfw: bool,
}

#[derive(Deserialize)]
pub struct RawFinalizeBody {
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

pub async fn handler_raw_upload_initial(
    State(s3_client): State<S3Client>,
    axum::extract::Query(params): axum::extract::Query<RawUploadInitialParams>,
    body: Body,
) -> Result<impl IntoResponse, Error> {
    // Collect the body data
    let body_data = body.collect().await.map_err(Error::Hyper)?.to_bytes();

    let mut pending_metadata = BTreeMap::new();
    pending_metadata.insert("_pending".to_string(), "true".to_string());
    pending_metadata.insert("_uploaded_at".to_string(), chrono::Utc::now().to_rfc3339());

    let ttl_hours = params.ttl_hours.unwrap_or(1);
    let expires = format!("+{}h", ttl_hours);

    if !params.is_nsfw {
        // For SFW videos, upload to both Storj (with TTL) and S3 (without TTL)
        let body_clone = body_data.clone();

        let storj_upload = upload_to_storj_with_ttl(
            &params.publisher_user_id,
            &params.video_id,
            &pending_metadata,
            &body_data,
            &expires,
            params.is_nsfw,
        );

        let s3_stream = futures_util::stream::once(async move { Ok(body_clone) });
        let s3_upload = upload_to_s3(
            &s3_client,
            &params.publisher_user_id,
            &params.video_id,
            &pending_metadata,
            s3_stream,
        );

        tokio::try_join!(storj_upload, s3_upload)?;
    } else {
        upload_to_storj_with_ttl(
            &params.publisher_user_id,
            &params.video_id,
            &pending_metadata,
            &body_data,
            &expires,
            params.is_nsfw,
        )
        .await?;
    }

    Ok(Json(json!({
        "status": "pending",
        "expires_in_hours": ttl_hours,
        "message": "Video uploaded successfully. Call /duplicate_raw/finalize to complete the upload."
    })))
}

pub async fn handler_raw_finalize(
    State(s3_client): State<S3Client>,
    axum::extract::Query(params): axum::extract::Query<RawFinalizeParams>,
    Json(body): Json<RawFinalizeBody>,
) -> Result<impl IntoResponse, Error> {
    let metadata = body.metadata;

    let (bucket, grant) = if params.is_nsfw {
        (YRAL_NSFW_VIDEOS.as_str(), ACCESS_GRANT_NSFW.as_str())
    } else {
        (YRAL_VIDEOS.as_str(), ACCESS_GRANT_SFW.as_str())
    };

    let src_path = format!(
        "sj://{}/{}/{}.mp4",
        bucket, params.publisher_user_id, params.video_id
    );

    // Download to temporary file
    let temp_file = format!(
        "/tmp/storj-finalize-{}-{}.mp4",
        params.publisher_user_id, params.video_id
    );

    let mut download_child = Command::new("uplink")
        .args([
            "cp",
            "--interactive=false",
            "--analytics=false",
            "--progress=false",
            "--access",
            grant,
            src_path.as_str(),
            temp_file.as_str(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()?;

    let download_status = download_child.wait().await?;
    if !download_status.success() {
        return Err(Error::Io(std::io::Error::other(format!(
            "Failed to download video from Storj for finalization: {}",
            download_status
        ))));
    }

    // Read the file data
    let file_data = tokio::fs::read(&temp_file).await?;

    // Re-upload with final metadata (no TTL)
    if !params.is_nsfw {
        // For SFW videos, upload to both Storj and S3
        let file_data_clone = file_data.clone();

        let storj_stream =
            futures_util::stream::once(async move { Ok::<_, reqwest::Error>(file_data.into()) });
        let s3_stream =
            futures_util::stream::once(
                async move { Ok::<_, reqwest::Error>(file_data_clone.into()) },
            );

        let storj_upload = upload_to_storj(
            &params.publisher_user_id,
            &params.video_id,
            &metadata,
            Box::pin(storj_stream),
            params.is_nsfw,
        );

        let s3_upload = upload_to_s3(
            &s3_client,
            &params.publisher_user_id,
            &params.video_id,
            &metadata,
            s3_stream,
        );

        tokio::try_join!(storj_upload, s3_upload)?;
    } else {
        // For NSFW videos, only upload to Storj
        let storj_stream =
            futures_util::stream::once(async move { Ok::<_, reqwest::Error>(file_data.into()) });

        upload_to_storj(
            &params.publisher_user_id,
            &params.video_id,
            &metadata,
            Box::pin(storj_stream),
            params.is_nsfw,
        )
        .await?;
    }

    // Clean up temp file
    tokio::fs::remove_file(&temp_file).await.ok();

    Ok(Json(json!({
        "status": "completed",
        "message": "Video finalized successfully with metadata."
    })))
}

async fn upload_to_storj_with_ttl(
    publisher_user_id: &str,
    video_id: &str,
    metadata: &BTreeMap<String, String>,
    body_data: &[u8],
    expires: &str,
    is_nsfw: bool,
) -> Result<(), Error> {
    let (bucket, grant) = if is_nsfw {
        (YRAL_NSFW_VIDEOS.as_str(), ACCESS_GRANT_NSFW.as_str())
    } else {
        (YRAL_VIDEOS.as_str(), ACCESS_GRANT_SFW.as_str())
    };
    let dest = format!("sj://{bucket}/{publisher_user_id}/{video_id}.mp4");

    let metadata_str = serde_json::to_string(metadata)
        .expect("serialization to go through as we are guaranteed utf-8");

    let mut child = Command::new("uplink")
        .args([
            "cp",
            "--interactive=false",
            "--analytics=false",
            "--progress=false",
            format!("--metadata={metadata_str}").as_str(),
            "--expires",
            expires,
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
    pipe.write_all(body_data).await?;
    pipe.flush().await?;
    drop(pipe);

    let status = child.wait().await?;
    if !status.success() {
        return Err(Error::Io(std::io::Error::other(format!(
            "uplink command failed with status: {status}"
        ))));
    }

    Ok(())
}
