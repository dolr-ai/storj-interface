use once_cell::sync::Lazy;
pub static YRAL_VIDEOS: Lazy<String> = Lazy::new(|| {
    const FALLBACK: &str = "yral-videos";
    std::env::var("SFW_BUCKET")
        .inspect_err(|err| println!("Using fallback for SFW_BUCKET because {err}"))
        .unwrap_or_else(|_| FALLBACK.into())
});
pub static YRAL_NSFW_VIDEOS: Lazy<String> = Lazy::new(|| {
    const FALLBACK: &str = "yral-nsfw-videos";
    std::env::var("NSFW_BUCKET")
        .inspect_err(|err| println!("Using fallback for NSFW_BUCKET because {err}"))
        .unwrap_or_else(|_| FALLBACK.into())
});
pub static ACCESS_GRANT_SFW: Lazy<String> = Lazy::new(|| {
    std::env::var("STORJ_ACCESS_GRANT_SFW")
        .expect("Access grant to be present: STORJ_ACCESS_GRANT_SFW")
});
pub static ACCESS_GRANT_NSFW: Lazy<String> = Lazy::new(|| {
    std::env::var("STORJ_ACCESS_GRANT_NSFW")
        .expect("Access grant to be present: STORJ_ACCESS_GRANT_NSFW")
});
pub static SERVICE_SECRET_TOKEN: Lazy<String> = Lazy::new(|| {
    format!(
        "Bearer {}",
        std::env::var("SERVICE_SECRET_TOKEN").expect("A shared secret to be present")
    )
});
