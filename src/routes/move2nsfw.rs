use axum::{response::IntoResponse, Json};
use reqwest::StatusCode;
use serde_json::json;
use sia_interface::move2nsfw::Args;

use crate::sia_client::{SiaClient, SiaError};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Sia(#[from] SiaError),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("err: {self}");
        let (status, message) = match self {
            Error::Sia(_) => (
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
    let sia_client = SiaClient::new();
    
    // Get bucket names
    let sfw_bucket = sia_client.get_bucket_name(false);
    let nsfw_bucket = sia_client.get_bucket_name(true);
    
    // Create object key
    let object_key = format!("{}/{}.mp4", request.publisher_user_id, request.video_id);
    
    // Move object from SFW to NSFW bucket
    sia_client.move_object(sfw_bucket, nsfw_bucket, &object_key).await?;
    
    Ok(Json(json!({
        "message": "Video moved to NSFW bucket successfully",
        "from_bucket": sfw_bucket,
        "to_bucket": nsfw_bucket,
        "key": object_key
    })))
}
