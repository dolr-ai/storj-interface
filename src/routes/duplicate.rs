use std::collections::BTreeMap;

/// Args for duplication request
pub struct Args {
    /// The publisher user principal supplied to off chain agent
    ///
    /// This used as directory key
    publisher_user_id: String,
    /// The video id on cloudflare
    ///
    /// This is used as object key
    video_id: String,
    /// Whether the video contains nsfw content
    ///
    /// This is used for segregation
    is_nsfw: bool,
    /// key-value pair to be added to video's metadata on storj
    metadata: BTreeMap<String, String>,
}
