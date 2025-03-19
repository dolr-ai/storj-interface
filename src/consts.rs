use once_cell::sync::Lazy;
pub const YRAL_VIDEOS: &str = "yral-videos";
pub const YRAL_NSFW_VIDEOS: &str = "yral-nsfw-videos";
pub static ACCESS_GRANT: Lazy<String> =
    Lazy::new(|| std::env::var("STORJ_ACCESS_GRANT").expect("Access grant to be present"));
