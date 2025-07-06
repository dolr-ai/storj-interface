use anyhow::Result;
use base64::{engine::general_purpose, Engine};
use reqwest::{header::HeaderMap, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use futures_util::Stream;
use bytes::Bytes;

use crate::consts::{
    RENTERD_BASE_URL, SFW_WALLETD_BASE_URL, NSFW_WALLETD_BASE_URL, 
    RENTERD_API_PASSWORD, SFW_WALLETD_API_PASSWORD, NSFW_WALLETD_API_PASSWORD,
    SFW_BUCKET, NSFW_BUCKET,
};

pub struct SiaClient {
    client: Client,
    renterd_base_url: String,
    sfw_walletd_base_url: String,
    nsfw_walletd_base_url: String,
    renterd_auth_header: String,
    sfw_walletd_auth_header: String,
    nsfw_walletd_auth_header: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketInfo {
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBucketRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub metadata: Option<BTreeMap<String, String>>,
}

#[derive(Debug, thiserror::Error)]
pub enum SiaError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Sia API error: {status} - {message}")]
    Api { status: StatusCode, message: String },
    #[error("Authentication failed")]
    Auth,
    #[error("Bucket not found: {0}")]
    BucketNotFound(String),
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    #[error("Object not found: {0}")]
    ObjectNotFound(String),
}

impl SiaClient {
    pub fn new() -> Self {
        let client = Client::new();
        
        // Create basic auth headers
        let renterd_auth = general_purpose::STANDARD
            .encode(format!(":{}", RENTERD_API_PASSWORD.as_str()));
        let sfw_walletd_auth = general_purpose::STANDARD
            .encode(format!(":{}", SFW_WALLETD_API_PASSWORD.as_str()));
        let nsfw_walletd_auth = general_purpose::STANDARD
            .encode(format!(":{}", NSFW_WALLETD_API_PASSWORD.as_str()));
        
        Self {
            client,
            renterd_base_url: RENTERD_BASE_URL.clone(),
            sfw_walletd_base_url: SFW_WALLETD_BASE_URL.clone(),
            nsfw_walletd_base_url: NSFW_WALLETD_BASE_URL.clone(),
            renterd_auth_header: format!("Basic {}", renterd_auth),
            sfw_walletd_auth_header: format!("Basic {}", sfw_walletd_auth),
            nsfw_walletd_auth_header: format!("Basic {}", nsfw_walletd_auth),
        }
    }

    // Bucket Management
    pub async fn create_bucket(&self, name: &str) -> Result<(), SiaError> {
        let url = format!("{}/bus/buckets", self.renterd_base_url);
        let request = CreateBucketRequest {
            name: name.to_string(),
        };
        
        let response = self.client
            .post(&url)
            .header("Authorization", &self.renterd_auth_header)
            .json(&request)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            Err(SiaError::Api {
                status,
                message,
            })
        }
    }

    pub async fn get_bucket(&self, name: &str) -> Result<BucketInfo, SiaError> {
        let url = format!("{}/bus/buckets/{}", self.renterd_base_url, name);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.renterd_auth_header)
            .send()
            .await?;
        
        if response.status() == StatusCode::NOT_FOUND {
            return Err(SiaError::BucketNotFound(name.to_string()));
        }
        
        if response.status().is_success() {
            let bucket: BucketInfo = response.json().await?;
            Ok(bucket)
        } else {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            Err(SiaError::Api {
                status,
                message,
            })
        }
    }

    // Object Storage Operations
    pub async fn upload_object<S>(&self, bucket: &str, key: &str, data: S, metadata: Option<BTreeMap<String, String>>) -> Result<(), SiaError>
    where
        S: Stream<Item = Result<Bytes, reqwest::Error>> + Send + Sync + 'static,
    {
        let url = format!("{}/worker/object/{}", self.renterd_base_url, key);
        
        let mut headers = HeaderMap::new();
        headers.insert("Authorization", self.renterd_auth_header.parse().unwrap());
        
        let mut query_params = vec![
            ("bucket", bucket.to_string()),
            ("minshards", "10".to_string()),
            ("totalshards", "30".to_string()),
        ];
        
        if let Some(meta) = metadata {
            let metadata_json = serde_json::to_string(&meta).unwrap();
            query_params.push(("metadata", metadata_json));
        }
        
        let response = self.client
            .put(&url)
            .headers(headers)
            .query(&query_params)
            .body(reqwest::Body::wrap_stream(data))
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            Err(SiaError::Api {
                status,
                message,
            })
        }
    }

    pub async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), SiaError> {
        let url = format!("{}/worker/object/{}", self.renterd_base_url, key);
        
        let response = self.client
            .delete(&url)
            .header("Authorization", &self.renterd_auth_header)
            .query(&[("bucket", bucket)])
            .send()
            .await?;
        
        if response.status() == StatusCode::NOT_FOUND {
            return Err(SiaError::ObjectNotFound(key.to_string()));
        }
        
        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            Err(SiaError::Api {
                status,
                message,
            })
        }
    }

    pub async fn download_object(&self, bucket: &str, key: &str) -> Result<reqwest::Response, SiaError> {
        let url = format!("{}/worker/object/{}", self.renterd_base_url, key);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.renterd_auth_header)
            .query(&[("bucket", bucket)])
            .send()
            .await?;
        
        if response.status() == StatusCode::NOT_FOUND {
            return Err(SiaError::ObjectNotFound(key.to_string()));
        }
        
        let status = response.status();
        if status.is_success() {
            Ok(response)
        } else {
            let message = response.text().await.unwrap_or_default();
            Err(SiaError::Api {
                status,
                message,
            })
        }
    }

    // Wallet Management
    pub async fn get_sfw_wallet(&self) -> Result<WalletInfo, SiaError> {
        let url = format!("{}/api/wallet", self.sfw_walletd_base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.sfw_walletd_auth_header)
            .send()
            .await?;
        
        if response.status() == StatusCode::NOT_FOUND {
            return Err(SiaError::WalletNotFound("SFW wallet".to_string()));
        }
        
        if response.status().is_success() {
            let wallet: WalletInfo = response.json().await?;
            Ok(wallet)
        } else {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            Err(SiaError::Api {
                status,
                message,
            })
        }
    }

    pub async fn get_nsfw_wallet(&self) -> Result<WalletInfo, SiaError> {
        let url = format!("{}/api/wallet", self.nsfw_walletd_base_url);
        
        let response = self.client
            .get(&url)
            .header("Authorization", &self.nsfw_walletd_auth_header)
            .send()
            .await?;
        
        if response.status() == StatusCode::NOT_FOUND {
            return Err(SiaError::WalletNotFound("NSFW wallet".to_string()));
        }
        
        if response.status().is_success() {
            let wallet: WalletInfo = response.json().await?;
            Ok(wallet)
        } else {
            let status = response.status();
            let message = response.text().await.unwrap_or_default();
            Err(SiaError::Api {
                status,
                message,
            })
        }
    }

    // Utility methods for content segregation

    pub fn get_bucket_name(&self, is_nsfw: bool) -> &str {
        if is_nsfw {
            &NSFW_BUCKET
        } else {
            &SFW_BUCKET
        }
    }

    // Initialize buckets if they don't exist
    pub async fn ensure_buckets_exist(&self) -> Result<(), SiaError> {
        // Try to get SFW bucket, create if it doesn't exist
        if let Err(SiaError::BucketNotFound(_)) = self.get_bucket(&SFW_BUCKET).await {
            self.create_bucket(&SFW_BUCKET).await?;
        }
        
        // Try to get NSFW bucket, create if it doesn't exist
        if let Err(SiaError::BucketNotFound(_)) = self.get_bucket(&NSFW_BUCKET).await {
            self.create_bucket(&NSFW_BUCKET).await?;
        }
        
        Ok(())
    }

    // Validate wallets exist
    pub async fn validate_wallets(&self) -> Result<(), SiaError> {
        self.get_sfw_wallet().await?;
        self.get_nsfw_wallet().await?;
        Ok(())
    }

    // Move object between buckets (for NSFW migration)
    pub async fn move_object(&self, source_bucket: &str, dest_bucket: &str, key: &str) -> Result<(), SiaError> {
        // Download from source
        let response = self.download_object(source_bucket, key).await?;
        let data = response.bytes_stream();
        
        // Upload to destination
        self.upload_object(dest_bucket, key, data, None).await?;
        
        // Delete from source
        self.delete_object(source_bucket, key).await?;
        
        Ok(())
    }
}