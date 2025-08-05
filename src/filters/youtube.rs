use lazy_static::lazy_static;
use regex::Regex;
use log::{debug, warn};
use crate::{
    models::{RawSearchResult, VideoInfo},
    error::VideoSearchError,
};

lazy_static! {
    static ref ID_REGEX: Regex = Regex::new(
        r"(?:youtube\.com/watch\?v=|youtu\.be/)([a-zA-Z0-9_-]{11})"
    ).unwrap();
    static ref SHORTS_REGEX: Regex = Regex::new(
        r"youtube\.com/shorts/([a-zA-Z0-9_-]{11})"
    ).unwrap();
}

/// YouTube video filter implementation
pub struct YouTubeFilter;

impl YouTubeFilter {
    fn extract_id(url: &str) -> Option<String> {
        ID_REGEX.captures(url)
            .or_else(|| SHORTS_REGEX.captures(url))
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string())
    }
    
    fn generate_thumbnails(id: &str) -> Vec<String> {
        vec![
            format!("https://img.youtube.com/vi/{}/default.jpg", id),
            format!("https://img.youtube.com/vi/{}/mqdefault.jpg", id),
            format!("https://img.youtube.com/vi/{}/hqdefault.jpg", id),
            format!("https://img.youtube.com/vi/{}/maxresdefault.jpg", id),
        ]
    }
}

impl super::LinkFilter for YouTubeFilter {
    fn filter(&self, results: &[RawSearchResult]) -> Result<Vec<VideoInfo>, VideoSearchError> {
        let videos: Vec<VideoInfo> = results
            .iter()
            .filter_map(|result| {
                let id = Self::extract_id(&result.url)?;
                
                debug!("Found YouTube video: ID={}, URL={}", id, result.url);
                
                Some(VideoInfo {
                    platform: "youtube".to_string(),
                    id: id.clone(),
                    title: result.title.clone(),
                    url: result.url.clone(),
                    thumbnails: Self::generate_thumbnails(&id),
                    source_domain: result.domain.clone(),
                })
            })
            .collect();
            
        if videos.is_empty() {
            warn!("No YouTube videos found in {} results", results.len());
            Err(VideoSearchError::NoResults)
        } else {
            Ok(videos)
        }
    }
    
    fn supported_domains(&self) -> &'static [&'static str] {
        &["youtube.com", "youtu.be"]
    }
}