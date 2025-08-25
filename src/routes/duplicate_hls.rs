use axum::{body::Bytes, extract::Multipart, response::IntoResponse, Json};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
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

    #[error("Invalid multipart data")]
    InvalidMultipart,

    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Invalid field value: {0}")]
    InvalidField(&'static str),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("err: {self}");
        let (status, message) = match self {
            Error::Network(_) | Error::Io(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error. Check server logs.",
            ),
            Error::InvalidMultipart | Error::MissingField(_) | Error::InvalidField(_) => {
                (StatusCode::BAD_REQUEST, "Invalid request data")
            }
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
pub struct HlsUploadMetadata {
    publisher_user_id: String,
    video_id: String,
    is_nsfw: bool,
    hls_file_name: String,
    #[serde(default)]
    metadata: BTreeMap<String, String>,
}

pub struct HlsUploadData {
    metadata: HlsUploadMetadata,
    file_data: Bytes,
}

async fn parse_multipart(mut multipart: Multipart) -> Result<HlsUploadData, Error> {
    let mut json_data = None;
    let mut file_data = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| Error::InvalidMultipart)?
    {
        let name = field.name().ok_or(Error::InvalidMultipart)?;

        match name {
            "data" => {
                let text = field.text().await.map_err(|_| Error::InvalidMultipart)?;
                json_data = Some(serde_json::from_str::<HlsUploadMetadata>(&text).map_err(
                    |_| Error::InvalidField("data must be valid JSON with required fields"),
                )?);
            }
            "file" => {
                file_data = Some(field.bytes().await.map_err(|_| Error::InvalidMultipart)?);
            }
            _ => {} // Ignore unknown fields
        }
    }

    let metadata = json_data.ok_or(Error::MissingField("data"))?;
    let file_data = file_data.ok_or(Error::MissingField("file"))?;

    if file_data.is_empty() {
        return Err(Error::MissingField("file"));
    }

    Ok(HlsUploadData {
        metadata,
        file_data,
    })
}

pub async fn handler(multipart: Multipart) -> Result<impl IntoResponse, Error> {
    let data = parse_multipart(multipart).await?;

    let (bucket, grant) = if data.metadata.is_nsfw {
        (YRAL_NSFW_VIDEOS.as_str(), ACCESS_GRANT_NSFW.as_str())
    } else {
        (YRAL_VIDEOS.as_str(), ACCESS_GRANT_SFW.as_str())
    };

    let dest = format!(
        "sj://{bucket}/{}/hls/{}",
        data.metadata.video_id, data.metadata.hls_file_name
    );

    let metadata_str = serde_json::to_string(&data.metadata.metadata)
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
        .spawn()?;

    let mut pipe = child.stdin.take().expect("Stdin pipe to be opened for us");

    // Write the file data to uplink
    pipe.write_all(&data.file_data).await?;
    pipe.flush().await?;
    drop(pipe); // Close stdin to signal EOF

    // Wait for uplink to complete
    let status = child.wait().await?;

    if !status.success() {
        return Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "uplink command failed",
        )));
    }

    Ok(())
}
