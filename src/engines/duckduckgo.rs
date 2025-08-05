use std::{time::{Duration, Instant}, sync::Arc};
use scraper::{Html, Selector, ElementRef};
use reqwest::{Client, header};
use url::Url;
use tokio::sync::Mutex;
use rand::Rng;
use log::warn;
use crate::{
    models::{RawSearchResult, PageParams},
    error::VideoSearchError,
};
use super::SearchEngine;

const DDG_URL: &str = "https://html.duckduckgo.com/html/";
const TIMEOUT: u64 = 10;
const MIN_REQUEST_DELAY: u64 = 2; // Минимальная задержка между запросами в секундах

/// DuckDuckGo search engine implementation with anti-bot protection
pub struct DuckDuckGo {
    client: Client,
    last_request: Arc<Mutex<Option<Instant>>>, // Трекер времени последнего запроса
}

impl DuckDuckGo {
    /// Create new DuckDuckGo search instance with bot protection
    pub fn new() -> Result<Self, VideoSearchError> {
        let mut headers = header::HeaderMap::new();
        headers.insert("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8".parse().unwrap());
        headers.insert("Accept-Language", "ru-RU,ru;q=0.8,en-US;q=0.5,en;q=0.3".parse().unwrap());
        headers.insert("Accept-Encoding", "gzip, deflate, br".parse().unwrap());
        headers.insert("Referer", "https://html.duckduckgo.com/".parse().unwrap());
        headers.insert("Origin", "https://html.duckduckgo.com".parse().unwrap());
        headers.insert("Upgrade-Insecure-Requests", "1".parse().unwrap());
        headers.insert("Sec-Fetch-Dest", "document".parse().unwrap());
        headers.insert("Sec-Fetch-Mode", "navigate".parse().unwrap());
        headers.insert("Sec-Fetch-Site", "same-origin".parse().unwrap());
        headers.insert("Sec-Fetch-User", "?1".parse().unwrap());

        Ok(Self {
            client: Client::builder()
                .default_headers(headers)
                .timeout(Duration::from_secs(TIMEOUT))
                .gzip(true)
                .brotli(true)
                .build()?,
            last_request: Arc::new(Mutex::new(None)),
        })
    }


    /// Добавляем случайную задержку между запросами
    async fn random_delay(&self) {
        let mut last_request = self.last_request.lock().await;
        if let Some(last) = *last_request {
            let elapsed = last.elapsed();
            let min_delay = Duration::from_secs(MIN_REQUEST_DELAY);
            let max_delay = Duration::from_secs(5);
                
            if elapsed < max_delay {
                let remaining = max_delay - elapsed;
                let delay = if elapsed < min_delay {
                    min_delay - elapsed + Duration::from_millis(rand::rng().random_range(100..500))
                } else {
                    Duration::from_millis(rand::rng().random_range(0..remaining.as_millis() as u64))
                };
                tokio::time::sleep(delay).await;
            }
        }
        *last_request = Some(Instant::now());
    }

    /// Генерируем случайный User-Agent для каждого запроса
    fn random_user_agent(&self) -> String {
        const USER_AGENTS: &[&str] = &[
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0",
            "Mozilla/5.0 (X11; Linux x86_64; rv:141.0) Gecko/20100101 Firefox/141.0",
        ];
        USER_AGENTS[rand::rng().random_range(0..USER_AGENTS.len())].to_string()
    }

    async fn fetch_page(
        &self,
        query: &str,
        params: Option<&PageParams>,
    ) -> Result<(Vec<RawSearchResult>, Option<PageParams>), VideoSearchError> {
        self.random_delay().await;

        let mut form_data = vec![
            ("q".to_string(), query.to_string()),
            ("kl".to_string(), "wt-wt".to_string()),
        ];
        
        if let Some(p) = params {
            form_data.push(("s".to_string(), p.offset.to_string()));
            form_data.push(("vqd".to_string(), p.page_token.clone()));
        }

        let response = self.client
            .post(DDG_URL)
            .header(header::USER_AGENT, self.random_user_agent())
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&form_data)
            .send()
            .await?
            .error_for_status()?;

        let html = response.text().await?;
        self.parse_page(&html)
    }

    fn parse_page(&self, html: &str) -> Result<(Vec<RawSearchResult>, Option<PageParams>), VideoSearchError> {
        let document = Html::parse_document(html);
        
        // Парсим все ссылки с результатами
        let link_selector = Selector::parse("a[href]").unwrap();
        let results: Vec<_> = document.select(&link_selector)
            .filter_map(|el| self.parse_link(el))
            .collect();
            
        // Парсим токен для следующей страницы
        let next_params = document
            .select(&Selector::parse("input[name=vqd]").unwrap())
            .next()
            .and_then(|e| e.value().attr("value"))
            .map(|vqd| PageParams {
                offset: 30,
                page_token: vqd.to_string(),
            });
            
        if results.is_empty() {
            warn!("Found links but no search results. HTML: {}", html);
            Err(VideoSearchError::NoResults)
        } else {
            Ok((results, next_params))
        }
    }

    fn parse_link(&self, element: ElementRef) -> Option<RawSearchResult> {
        let href = element.value().attr("href")?.to_string();
        
        // Фильтруем только внешние ссылки (исключаем внутренние ссылки DDG)
        if href.starts_with("/") || href.starts_with("?") || href.contains("duckduckgo.com") {
            return None;
        }
        
        // Извлекаем домен
        let domain = Url::parse(&href)
            .ok()
            .and_then(|u| u.host().map(|h| h.to_string()))
            .unwrap_or_else(|| href.split('/').nth(2).unwrap_or("").to_string());
        
        // Получаем текст ссылки и текст вокруг нее
        let title = element.text().collect::<String>().trim().to_string();
        let description = element.parent()?
            .next_sibling()
            .and_then(|n| n.value().as_text())
            .map(|t| t.text.trim().to_string())
            .unwrap_or_default();

        Some(RawSearchResult {
            title,
            url: href,
            description,
            domain,
        })
    }
}

#[async_trait::async_trait]
impl SearchEngine for DuckDuckGo {
    async fn search(
        &self,
        query: &str,
        max_pages: usize,
    ) -> Result<Vec<RawSearchResult>, VideoSearchError> {
        let mut all_results = Vec::with_capacity(max_pages * 30);
        let mut next_params = None;
        let mut pages_fetched = 0;
        
        loop {
            let (results, params) = self.fetch_page(query, next_params.as_ref()).await?;
            all_results.extend(results);
            pages_fetched += 1;
            
            // Stop if no more pages or reached max pages
            if params.is_none() || pages_fetched >= max_pages {
                break;
            }
            
            next_params = params;
        }
        
        if all_results.is_empty() {
            warn!("No results found for query: {}", query);
            Err(VideoSearchError::NoResults)
        } else {
            Ok(all_results)
        }
    }
    
    fn name(&self) -> &'static str {
        "duckduckgo"
    }
}