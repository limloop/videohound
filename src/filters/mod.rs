//! Video filters module

mod youtube;

pub use youtube::YouTubeFilter;

use crate::{models::{RawSearchResult, VideoInfo}, error::VideoSearchError};

/// Common interface for all filters
pub trait LinkFilter: Send + Sync {
    /// Filter raw results into video info
    fn filter(&self, results: &[RawSearchResult]) -> Result<Vec<VideoInfo>, VideoSearchError>;
    
    /// Supported domains
    fn supported_domains(&self) -> &'static [&'static str];
}