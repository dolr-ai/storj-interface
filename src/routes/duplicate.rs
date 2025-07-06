use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sia_interface::duplicate::Args;

use crate::sia_client::{SiaClient, SiaError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error(transparent)]
    Sia(#[from] SiaError),

    #[error("Cloudflare returned non-ok status ({0}) when fetching the video")]
    Clouflare(StatusCode),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("err: {self}");
        let (status, message) = match self {
            Error::Network(_) | Error::Sia(_) => (
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

pub async fn handler(
    Json(Args {
        publisher_user_id,
        video_id,
        is_nsfw,
        metadata,
    }): Json<Args>,
) -> Result<impl IntoResponse, Error> {
    let sia_client = SiaClient::new();
    
    // Determine the bucket based on content type
    let bucket = sia_client.get_bucket_name(is_nsfw);
    
    // Create object key
    let object_key = format!("{}/{}.mp4", publisher_user_id, video_id);
    
    // Download video from Cloudflare
    let source = format!(
        "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com/{}/downloads/default.mp4",
        video_id
    );
    
    let req = reqwest::get(source).await?;
    let status = req.status();
    
    if status != StatusCode::OK {
        return Err(Error::Clouflare(status));
    }
    
    // Get the byte stream
    let stream = req.bytes_stream();
    
    // Upload to Sia
    sia_client.upload_object(bucket, &object_key, stream, Some(metadata)).await?;
    
    Ok(Json(json!({
        "message": "Video uploaded successfully",
        "bucket": bucket,
        "key": object_key
    })))
}
