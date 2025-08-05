//! Search engines module

mod duckduckgo;

pub use duckduckgo::DuckDuckGo;

use async_trait::async_trait;
use crate::{models::RawSearchResult, error::VideoSearchError};

/// Common interface for all search engines
#[async_trait]
pub trait SearchEngine: Send + Sync {
    /// Perform search with pagination
    async fn search(
        &self,
        query: &str,
        max_pages: usize,
    ) -> Result<Vec<RawSearchResult>, VideoSearchError>;
    
    /// Engine name
    fn name(&self) -> &'static str;
}