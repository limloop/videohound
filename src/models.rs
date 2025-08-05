use serde::{Serialize, Deserialize};

/// Raw search result from engine
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawSearchResult {
    /// Video title
    pub title: String,
    /// Direct URL
    pub url: String,
    /// Short description
    pub description: String,
    /// Source domain
    pub domain: String,
}

/// Processed video information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoInfo {
    /// Platform name (youtube, vimeo, etc)
    pub platform: String,
    /// Unique video ID
    pub id: String,
    /// Video title
    pub title: String,
    /// Direct URL
    pub url: String,
    /// Available thumbnails (different resolutions)
    pub thumbnails: Vec<String>,
    /// Original domain where found
    pub source_domain: String,
}

/// Pagination parameters
#[derive(Debug, Clone)]
pub struct PageParams {
    /// Results offset
    pub offset: usize,
    /// Platform-specific token
    pub page_token: String,
}