use axum::{response::IntoResponse, Extension, Json};
use futures_util::StreamExt;
use reqwest::StatusCode;
use serde_json::json;
use storj_interface::duplicate::Args;

use crate::consts::{RENTERD_API_URL_NSFW, RENTERD_API_URL_SFW, SIA_BUCKET_NSFW, SIA_BUCKET_SFW};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("Cloudflare returned non-ok status ({0}) when fetching the video")]
    Cloudflare(StatusCode),

    #[error("Renterd returned non-ok status ({0})")]
    Renterd(StatusCode),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("err: {self}");
        let (status, message) = match self {
            Error::Network(_) | Error::Io(_) | Error::Json(_) | Error::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error. Check server logs.",
            ),
            Error::Cloudflare(StatusCode::NOT_FOUND) => (
                StatusCode::NOT_FOUND,
                "The video doesn't exist on cloudflare",
            ),
            Error::Cloudflare(_) => (
                StatusCode::BAD_REQUEST,
                "The video couldn't be fetched from cloudflare. Check server logs.",
            ),
            Error::Renterd(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to upload to Sia. Check server logs.",
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
    Extension(token_cache): Extension<super::SiaTokenCache>,
    Json(Args {
        publisher_user_id,
        video_id,
        is_nsfw,
        metadata,
    }): Json<Args>,
) -> Result<impl IntoResponse, Error> {
    let (bucket, base_url) = if is_nsfw {
        (SIA_BUCKET_NSFW, RENTERD_API_URL_NSFW)
    } else {
        (SIA_BUCKET_SFW, RENTERD_API_URL_SFW)
    };

    // Download from Cloudflare
    let source = format!(
        "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com/{}/downloads/default.mp4",
        video_id
    );

    let req = reqwest::get(&source).await?;
    let status = req.status();

    if status != StatusCode::OK {
        return Err(Error::Cloudflare(status));
    }

    // Get auth token for renterd
    let token = super::get_auth_token(base_url, &token_cache)
        .await
        .map_err(|e| Error::Internal(e.to_string()))?;
    let cookie = format!("renterd_auth={}", token);

    // Prepare the upload URL
    let object_key = format!("{}/{}.mp4", publisher_user_id, video_id);
    let upload_url = format!(
        "{}/api/worker/object/{}?bucket={}",
        base_url,
        urlencoding::encode(&object_key),
        bucket
    );

    // Create a new client for the upload
    let client = reqwest::Client::new();

    // Upload to Sia with streaming
    let mut stream = req.bytes_stream();
    let mut body_bytes = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        body_bytes.extend_from_slice(&chunk);
    }

    let upload_response = client
        .put(&upload_url)
        .header("Cookie", &cookie)
        .header("Content-Type", "video/mp4")
        .header("X-Metadata", serde_json::to_string(&metadata)?)
        .body(body_bytes)
        .send()
        .await?;

    if !upload_response.status().is_success() {
        return Err(Error::Renterd(upload_response.status()));
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Video uploaded to Sia successfully",
            "bucket": bucket,
            "key": object_key
        })),
    ))
}
