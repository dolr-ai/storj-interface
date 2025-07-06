use once_cell::sync::Lazy;

// Bucket names for SFW and NSFW content
pub static SFW_BUCKET: Lazy<String> = Lazy::new(|| {
    const FALLBACK: &str = "yral-videos";
    std::env::var("SFW_BUCKET")
        .inspect_err(|err| println!("Using fallback for SFW_BUCKET because {err}"))
        .unwrap_or_else(|_| FALLBACK.into())
});
pub static NSFW_BUCKET: Lazy<String> = Lazy::new(|| {
    const FALLBACK: &str = "yral-nsfw-videos";
    std::env::var("NSFW_BUCKET")
        .inspect_err(|err| println!("Using fallback for NSFW_BUCKET because {err}"))
        .unwrap_or_else(|_| FALLBACK.into())
});

// Sia service endpoints
pub static RENTERD_BASE_URL: Lazy<String> = Lazy::new(|| {
    const FALLBACK: &str = "http://localhost:9980";
    std::env::var("RENTERD_BASE_URL")
        .inspect_err(|err| println!("Using fallback for RENTERD_BASE_URL because {err}"))
        .unwrap_or_else(|_| FALLBACK.into())
});
pub static SFW_WALLETD_BASE_URL: Lazy<String> = Lazy::new(|| {
    const FALLBACK: &str = "http://localhost:9983";
    std::env::var("SFW_WALLETD_BASE_URL")
        .inspect_err(|err| println!("Using fallback for SFW_WALLETD_BASE_URL because {err}"))
        .unwrap_or_else(|_| FALLBACK.into())
});
pub static NSFW_WALLETD_BASE_URL: Lazy<String> = Lazy::new(|| {
    const FALLBACK: &str = "http://localhost:9984";
    std::env::var("NSFW_WALLETD_BASE_URL")
        .inspect_err(|err| println!("Using fallback for NSFW_WALLETD_BASE_URL because {err}"))
        .unwrap_or_else(|_| FALLBACK.into())
});

// Sia authentication
pub static RENTERD_API_PASSWORD: Lazy<String> = Lazy::new(|| {
    std::env::var("RENTERD_API_PASSWORD")
        .expect("Renterd API password to be present: RENTERD_API_PASSWORD")
});
pub static SFW_WALLETD_API_PASSWORD: Lazy<String> = Lazy::new(|| {
    std::env::var("SFW_WALLETD_API_PASSWORD")
        .expect("SFW Walletd API password to be present: SFW_WALLETD_API_PASSWORD")
});
pub static NSFW_WALLETD_API_PASSWORD: Lazy<String> = Lazy::new(|| {
    std::env::var("NSFW_WALLETD_API_PASSWORD")
        .expect("NSFW Walletd API password to be present: NSFW_WALLETD_API_PASSWORD")
});

// Service authentication
pub static SERVICE_SECRET_TOKEN: Lazy<String> = Lazy::new(|| {
    format!(
        "Bearer {}",
        std::env::var("SERVICE_SECRET_TOKEN").expect("A shared secret to be present")
    )
});
