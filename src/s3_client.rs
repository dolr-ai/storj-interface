use aws_config::meta::region::RegionProviderChain;
use aws_config::Region;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::{Client, Config};
use bytes::Bytes;
use futures_util::StreamExt;
use std::collections::HashMap;

use crate::consts::{
    HETZNER_S3_ACCESS_KEY, HETZNER_S3_BUCKET, HETZNER_S3_ENDPOINT, HETZNER_S3_REGION,
    HETZNER_S3_SECRET_KEY,
};

#[derive(Clone)]
pub struct S3Client {
    client: Client,
    bucket: String,
}

impl S3Client {
    pub async fn new() -> Self {
        let region_provider =
            RegionProviderChain::default_provider().or_else(Region::new(HETZNER_S3_REGION.clone()));

        let creds = Credentials::new(
            HETZNER_S3_ACCESS_KEY.as_str(),
            HETZNER_S3_SECRET_KEY.as_str(),
            None,
            None,
            "hetzner_s3",
        );

        let config = Config::builder()
            .region(
                region_provider
                    .region()
                    .await
                    .unwrap_or_else(|| Region::new("eu-central")),
            )
            .endpoint_url(HETZNER_S3_ENDPOINT.as_str())
            .credentials_provider(creds)
            .force_path_style(true)
            .build();

        let client = Client::from_conf(config);

        Self {
            client,
            bucket: HETZNER_S3_BUCKET.clone(),
        }
    }

    pub async fn upload_video_stream(
        &self,
        key: &str,
        stream: impl futures_util::Stream<Item = Result<Bytes, reqwest::Error>>,
        metadata: &HashMap<String, String>,
    ) -> Result<(), aws_sdk_s3::Error> {
        let chunks: Vec<Bytes> = stream
            .filter_map(|chunk| async move { chunk.ok() })
            .collect()
            .await;

        let body_bytes = chunks.concat();
        let body = ByteStream::from(body_bytes);

        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_type("video/mp4");

        // Add metadata
        for (k, v) in metadata {
            request = request.metadata(k, v);
        }

        request.send().await?;
        Ok(())
    }

    pub async fn upload_hls_segment(
        &self,
        key: &str,
        data: Bytes,
        metadata: &HashMap<String, String>,
    ) -> Result<(), aws_sdk_s3::Error> {
        let content_type = if key.ends_with(".m3u8") {
            "application/vnd.apple.mpegurl"
        } else if key.ends_with(".ts") {
            "video/mp2t"
        } else {
            "application/octet-stream"
        };

        let mut request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type);

        // Add metadata
        for (k, v) in metadata {
            request = request.metadata(k, v);
        }

        request.send().await?;
        Ok(())
    }

    pub async fn download_video(&self, key: &str) -> Result<Vec<u8>, String> {
        let resp = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let data = resp.body.collect().await.map_err(|e| e.to_string())?;
        Ok(data.into_bytes().to_vec())
    }

    pub async fn delete_video(&self, key: &str) -> Result<(), aws_sdk_s3::Error> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await?;
        Ok(())
    }
}
