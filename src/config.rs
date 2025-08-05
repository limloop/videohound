use serde::{Serialize, Deserialize};

/// Search configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchConfig {
    /// Max pages to fetch
    pub max_pages: usize,
    /// Timeout in seconds
    pub timeout: u64
}