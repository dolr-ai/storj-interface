use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::{header::AUTHORIZATION, StatusCode};
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration, Instant};

use crate::consts::{
    RENTERD_API_PASSWORD, RENTERD_API_URL_NSFW, RENTERD_API_URL_SFW, SIA_BUCKET_NSFW,
    SIA_BUCKET_SFW,
};

pub mod duplicate;
pub mod move2nsfw;

#[derive(Clone)]
pub struct SiaTokenCache {
    pub tokens: Arc<RwLock<SiaTokens>>,
}

#[derive(Clone, Default)]
pub struct SiaTokens {
    pub sfw: Option<CachedToken>,
    pub nsfw: Option<CachedToken>,
}

#[derive(Clone)]
pub struct CachedToken {
    pub token: String,
    pub expires_at: Instant,
}

impl SiaTokenCache {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(SiaTokens::default())),
        }
    }
}

#[derive(serde::Deserialize)]
struct AuthResponse {
    token: String,
}

#[derive(Serialize)]
struct CreateBucketRequest {
    name: String,
    policy: BucketPolicy,
}

#[derive(Serialize)]
struct BucketPolicy {
    #[serde(rename = "publicReadAccess")]
    public_read_access: bool,
}

pub async fn get_auth_token(
    base_url: &str,
    cache: &SiaTokenCache,
) -> Result<String, Box<dyn std::error::Error>> {
    // Determine which token to use
    let is_sfw = base_url.contains("9980");

    // Check if we have a valid cached token
    {
        let tokens = cache.tokens.read().await;
        let cached = if is_sfw { &tokens.sfw } else { &tokens.nsfw };

        if let Some(cached_token) = cached {
            if Instant::now() < cached_token.expires_at {
                return Ok(cached_token.token.clone());
            }
        }
    }

    // Token is expired or doesn't exist, get a new one
    let client = reqwest::Client::new();
    let auth = BASE64.encode(format!(":{}", RENTERD_API_PASSWORD));

    let response = client
        .post(format!("{}/api/auth?validity=3600000", base_url))
        .header(AUTHORIZATION, format!("Basic {}", auth))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Failed to get auth token: HTTP {}", response.status()).into());
    }

    let auth_response: AuthResponse = response.json().await?;
    let new_token = auth_response.token;

    // Cache the new token with expiration (50 minutes to be safe with 60 minute validity)
    let cached_token = CachedToken {
        token: new_token.clone(),
        expires_at: Instant::now() + Duration::from_secs(50 * 60),
    };

    let mut tokens = cache.tokens.write().await;
    if is_sfw {
        tokens.sfw = Some(cached_token);
    } else {
        tokens.nsfw = Some(cached_token);
    }

    Ok(new_token)
}

pub async fn create_bucket_if_not_exists(
    base_url: &str,
    bucket_name: &str,
    cache: &SiaTokenCache,
) -> Result<(), Box<dyn std::error::Error>> {
    let token = get_auth_token(base_url, cache).await?;
    let client = reqwest::Client::new();

    // Set the auth cookie
    let cookie = format!("renterd_auth={}", token);

    // First, check if bucket exists
    let check_url = format!("{}/api/bus/bucket/{}", base_url, bucket_name);
    let check_response = client
        .get(&check_url)
        .header("Cookie", &cookie)
        .send()
        .await?;

    match check_response.status() {
        StatusCode::OK => {
            // Bucket already exists
            println!("✓ Bucket '{}' already exists", bucket_name);
            Ok(())
        }
        StatusCode::NOT_FOUND => {
            // Bucket doesn't exist, create it
            println!("Creating bucket '{}'...", bucket_name);

            let create_url = format!("{}/api/bus/buckets", base_url);
            let create_request = CreateBucketRequest {
                name: bucket_name.to_string(),
                policy: BucketPolicy {
                    public_read_access: false,
                },
            };

            let create_response = client
                .post(&create_url)
                .header("Cookie", &cookie)
                .json(&create_request)
                .send()
                .await?;

            if !create_response.status().is_success() {
                return Err(
                    format!("Failed to create bucket: HTTP {}", create_response.status()).into(),
                );
            }

            println!("✓ Bucket '{}' created successfully", bucket_name);
            Ok(())
        }
        status => Err(format!("Failed to check bucket: HTTP {}", status).into()),
    }
}

async fn download_renterd() -> Result<(), Box<dyn std::error::Error>> {
    let renterd_version = "v2.5.0";
    let renterd_dir = "./renterd-bin";

    // Create directory if it doesn't exist
    fs::create_dir_all(renterd_dir)?;

    // For Linux AMD64 only
    let download_url = format!(
        "https://github.com/SiaFoundation/renterd/releases/download/{}/renterd_linux_amd64.zip",
        renterd_version
    );

    println!("Downloading renterd {} for Linux AMD64...", renterd_version);
    println!("URL: {}", download_url);

    // Download the file
    let response = reqwest::get(&download_url).await?;
    if !response.status().is_success() {
        return Err(format!("Failed to download renterd: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes().await?;
    println!("Downloaded {} bytes", bytes.len());

    let zip_path = format!("{}/renterd.zip", renterd_dir);
    let mut file = fs::File::create(&zip_path)?;
    file.write_all(&bytes)?;
    file.flush()?;
    drop(file);

    println!("Saved zip file to: {}", zip_path);

    // Extract the zip file
    println!("Extracting renterd...");
    let status = Command::new("unzip")
        .arg("-o")
        .arg("renterd.zip")
        .current_dir(renterd_dir)
        .status()
        .await?;

    if !status.success() {
        return Err("Failed to extract renterd".into());
    }

    // Make executable (Linux)
    use std::os::unix::fs::PermissionsExt;
    let renterd_path = format!("{}/renterd", renterd_dir);
    let mut perms = fs::metadata(&renterd_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&renterd_path, perms)?;

    // Clean up zip file
    fs::remove_file(&zip_path)?;

    println!("renterd downloaded successfully to {}", renterd_dir);
    Ok(())
}

pub async fn init() {
    println!("Initializing renterd instances...");

    // Check if renterd binary exists, download if not
    let renterd_path = "./renterd-bin/renterd";
    if !Path::new(renterd_path).exists() {
        println!("renterd binary not found, downloading...");

        match download_renterd().await {
            Ok(_) => println!("renterd downloaded successfully"),
            Err(e) => {
                eprintln!("Failed to download renterd: {}", e);
                return;
            }
        }
    }

    // Create directories for each instance
    fs::create_dir_all("./data/renterd-sfw").expect("Failed to create SFW data directory");
    fs::create_dir_all("./data/renterd-nsfw").expect("Failed to create NSFW data directory");

    // Get environment variables
    let api_password = std::env::var("RENTERD_API_PASSWORD").unwrap_or_else(|_| "1234".to_string());
    let sfw_seed = std::env::var("SIA_SEED_SFW").unwrap_or_else(|_| {
        eprintln!("Warning: SIA_SEED_SFW not set, using test seed");
        "test seed phrase for sfw instance replace this with real seed phrase for production"
            .to_string()
    });
    let nsfw_seed = std::env::var("SIA_SEED_NSFW").unwrap_or_else(|_| {
        eprintln!("Warning: SIA_SEED_NSFW not set, using test seed");
        "test seed phrase for nsfw instance replace this with real seed phrase for production"
            .to_string()
    });

    // Start SFW renterd instance
    let renterd_path_sfw = renterd_path.to_string();
    let api_password_sfw = api_password.clone();
    tokio::spawn(async move {
        println!("Starting SFW renterd on port 9980...");

        let mut cmd = Command::new(&renterd_path_sfw)
            .arg("-http=:9980")
            .arg("-bus.gatewayAddr=:9881")
            .arg("-s3.address=:8080")
            .arg("-dir=./data/renterd-sfw")
            .arg("-db.name=renterd_sfw")
            .arg("-db.metricsName=renterd_metrics_sfw")
            .arg("-openui=false")
            .env("RENTERD_API_PASSWORD", &api_password_sfw)
            .env("RENTERD_WALLET_SEED", sfw_seed)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start SFW renterd");

        // Wait for the process
        let status = cmd
            .wait()
            .await
            .expect("SFW renterd process wasn't running");
        eprintln!("SFW renterd exited with: {}", status);
    });

    // Give first instance time to start
    sleep(Duration::from_secs(2)).await;

    // Start NSFW renterd instance
    let renterd_path_nsfw = renterd_path.to_string();
    let api_password_nsfw = api_password;
    tokio::spawn(async move {
        println!("Starting NSFW renterd on port 9981...");

        let mut cmd = Command::new(&renterd_path_nsfw)
            .arg("-http=:9981")
            .arg("-bus.gatewayAddr=:9882")
            .arg("-s3.address=:8081")
            .arg("-dir=./data/renterd-nsfw")
            .arg("-db.name=renterd_nsfw")
            .arg("-db.metricsName=renterd_metrics_nsfw")
            .arg("-openui=false")
            .env("RENTERD_API_PASSWORD", api_password_nsfw)
            .env("RENTERD_WALLET_SEED", nsfw_seed)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start NSFW renterd");

        // Wait for the process
        let status = cmd
            .wait()
            .await
            .expect("NSFW renterd process wasn't running");
        eprintln!("NSFW renterd exited with: {}", status);
    });

    // Give renterd instances time to start
    sleep(Duration::from_secs(5)).await;

    println!("Renterd instances initialization complete");
    println!("SFW instance: http://localhost:9980");
    println!("NSFW instance: http://localhost:9981");

    // Create buckets if they don't exist
    println!("\nChecking and creating buckets...");

    // Create a token cache for initialization
    let cache = SiaTokenCache::new();

    // Create SFW bucket
    if let Err(e) = create_bucket_if_not_exists(RENTERD_API_URL_SFW, SIA_BUCKET_SFW, &cache).await {
        eprintln!("Failed to create SFW bucket: {}", e);
    }

    // Create NSFW bucket
    if let Err(e) = create_bucket_if_not_exists(RENTERD_API_URL_NSFW, SIA_BUCKET_NSFW, &cache).await
    {
        eprintln!("Failed to create NSFW bucket: {}", e);
    }

    println!("\nInitialization complete!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download_renterd() {
        // Clean up any existing test directory
        let _ = fs::remove_dir_all("./renterd-bin");

        // Test downloading renterd
        match download_renterd().await {
            Ok(_) => {
                // Check if file exists
                let renterd_path = "./renterd-bin/renterd";
                if std::path::Path::new(renterd_path).exists() {
                    println!("✓ renterd binary exists at {}", renterd_path);

                    // Check file size
                    if let Ok(metadata) = fs::metadata(renterd_path) {
                        println!("✓ File size: {} bytes", metadata.len());
                    }
                } else {
                    // List what's in the directory
                    if let Ok(entries) = fs::read_dir("./renterd-bin") {
                        println!("Files in renterd-bin:");
                        for entry in entries {
                            if let Ok(entry) = entry {
                                println!("  - {}", entry.file_name().to_string_lossy());
                            }
                        }
                    }
                    panic!("renterd binary not found after download");
                }

                // Clean up test directory after
                let _ = fs::remove_dir_all("./renterd-bin");
            }
            Err(e) => {
                panic!("Failed to download renterd: {}", e);
            }
        }
    }
}
