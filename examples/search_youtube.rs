use videohound::{VideoSearcher, VideoHound, engines::DuckDuckGo, filters::YouTubeFilter, SearchConfig};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let searcher = VideoHound::new(
        DuckDuckGo::new()?,
        YouTubeFilter
    );

    let config = SearchConfig {
        max_pages: 1,
        timeout: 30,
    };

    let results = searcher.search("rust programming tutorial size:www.youtube.com/watch", &config).await?;

    println!("Найдено видео: {}", results.len());
    for (i, video) in results.iter().enumerate() {
        println!("\n{}. {}", i + 1, video.title);
        println!("   Ссылка: {}", video.url);
        println!("   Превью: {}", video.thumbnails[0]);
    }

    Ok(())
}