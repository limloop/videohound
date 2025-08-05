use thiserror::Error;

/// All possible library errors
#[derive(Error, Debug)]
pub enum VideoSearchError {
    /// Network-related errors
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    /// HTML parsing failed
    #[error("Failed to parse HTML content")]
    ParseError,
    
    /// No results found
    #[error("No search results found")]
    NoResults,
    
    /// Pagination issues
    #[error("Pagination error: {0}")]
    PaginationError(String),
    
    /// Invalid URL format
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    
    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Platform-specific errors
    #[error("Platform error: {0}")]
    PlatformError(String),
}