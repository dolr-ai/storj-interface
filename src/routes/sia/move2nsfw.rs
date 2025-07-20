use axum::{response::IntoResponse, Extension, Json};
use futures_util::StreamExt;
use reqwest::StatusCode;
use serde_json::json;
use storj_interface::move2nsfw::Args;

use crate::consts::{RENTERD_API_URL_NSFW, RENTERD_API_URL_SFW, SIA_BUCKET_NSFW, SIA_BUCKET_SFW};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Network(#[from] reqwest::Error),

    #[error("Renterd returned non-ok status ({0})")]
    Renterd(StatusCode),

    #[error("Object not found in source bucket")]
    NotFound,

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("err: {self}");
        let (status, message) = match self {
            Error::Network(_) | Error::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error. Check server logs.",
            ),
            Error::Renterd(status) if status == StatusCode::NOT_FOUND => {
                (StatusCode::NOT_FOUND, "Video not found in SFW bucket")
            }
            Error::Renterd(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to move object in Sia. Check server logs.",
            ),
            Error::NotFound => (StatusCode::NOT_FOUND, "Video not found in SFW bucket"),
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
    Json(request): Json<Args>,
) -> Result<impl IntoResponse, Error> {
    let object_key = format!("{}/{}.mp4", request.publisher_user_id, request.video_id);

    // Get auth tokens for both instances
    let sfw_token = super::get_auth_token(RENTERD_API_URL_SFW, &token_cache)
        .await
        .map_err(|e| Error::Internal(e.to_string()))?;
    let nsfw_token = super::get_auth_token(RENTERD_API_URL_NSFW, &token_cache)
        .await
        .map_err(|e| Error::Internal(e.to_string()))?;
    let sfw_cookie = format!("renterd_auth={}", sfw_token);
    let nsfw_cookie = format!("renterd_auth={}", nsfw_token);

    let client = reqwest::Client::new();

    // Step 1: Download from SFW bucket
    let download_url = format!(
        "{}/api/worker/object/{}?bucket={}",
        RENTERD_API_URL_SFW,
        urlencoding::encode(&object_key),
        SIA_BUCKET_SFW
    );

    let download_response = client
        .get(&download_url)
        .header("Cookie", &sfw_cookie)
        .send()
        .await?;

    if download_response.status() == StatusCode::NOT_FOUND {
        return Err(Error::NotFound);
    }

    if !download_response.status().is_success() {
        return Err(Error::Renterd(download_response.status()));
    }

    // Collect the data
    let mut stream = download_response.bytes_stream();
    let mut body_bytes = Vec::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        body_bytes.extend_from_slice(&chunk);
    }

    // Step 2: Upload to NSFW bucket
    let upload_url = format!(
        "{}/api/worker/object/{}?bucket={}",
        RENTERD_API_URL_NSFW,
        urlencoding::encode(&object_key),
        SIA_BUCKET_NSFW
    );

    let upload_response = client
        .put(&upload_url)
        .header("Cookie", &nsfw_cookie)
        .header("Content-Type", "video/mp4")
        .body(body_bytes)
        .send()
        .await?;

    if !upload_response.status().is_success() {
        return Err(Error::Renterd(upload_response.status()));
    }

    // Step 3: Delete from SFW bucket
    let delete_response = client
        .delete(&download_url)
        .header("Cookie", &sfw_cookie)
        .send()
        .await?;

    if !delete_response.status().is_success() {
        // Log the error but don't fail the operation
        println!(
            "Warning: Failed to delete from SFW bucket: {}",
            delete_response.status()
        );
    }

    Ok((
        StatusCode::OK,
        Json(json!({
            "message": "Video moved to NSFW bucket successfully",
            "key": object_key
        })),
    ))
}
