pub mod engines;
pub mod filters;
pub mod models;
pub mod error;
pub mod config;

pub use models::VideoInfo;
pub use error::VideoSearchError;
pub use config::SearchConfig;

use async_trait::async_trait;

/// Main search interface
#[async_trait]
pub trait VideoSearcher {
    /// Search videos with given query and configuration
    async fn search(
        &self,
        query: &str,
        config: &SearchConfig,
    ) -> Result<Vec<VideoInfo>, VideoSearchError>;
}

/// Default implementation combining engine and filter
pub struct VideoHound<E, F> {
    engine: E,
    filter: F,
}

impl<E, F> VideoHound<E, F> {
    /// Create new instance with specified engine and filter
    pub fn new(engine: E, filter: F) -> Self {
        Self { engine, filter }
    }
}

#[async_trait]
impl<E, F> VideoSearcher for VideoHound<E, F>
where
    E: engines::SearchEngine + Send + Sync,
    F: filters::LinkFilter + Send + Sync,
{
    async fn search(
        &self,
        query: &str,
        config: &SearchConfig,
    ) -> Result<Vec<VideoInfo>, VideoSearchError> {
        let raw_results = self.engine.search(query, config.max_pages).await?;
        self.filter.filter(&raw_results)
    }
}