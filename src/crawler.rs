use anyhow::Result;
use backoff::ExponentialBackoff;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;

#[derive(Debug, Serialize)]
pub struct DocPage {
    pub title: String,
    pub content: String,
    pub url: String,
    pub related_links: Vec<String>,
}

pub struct CrawlerConfig {
    pub max_retries: u32,
    pub concurrency: usize,
    pub timeout: std::time::Duration,
}

pub struct Crawler {
    client: Client,
    config: CrawlerConfig,
    visited_urls: Arc<Mutex<HashSet<String>>>,
}

impl Crawler {
    pub fn new(config: CrawlerConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            visited_urls: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub async fn crawl_url(&mut self, url: &str, recursive: bool) -> Result<Vec<DocPage>> {
        let mut pages = Vec::new();
        
        {
            let mut visited = self.visited_urls.lock().await;
            if visited.contains(url) {
                return Ok(pages);
            }
            visited.insert(url.to_string());
        }
        
        let page = self.fetch_page(url).await?;
        pages.push(page);

        if recursive {
            let links: Vec<String> = pages[0].related_links.clone();
            let client = self.client.clone();
            let visited_urls = self.visited_urls.clone();
            
            let results: Vec<_> = stream::iter(links)
                .map(|link| {
                    let client = client.clone();
                    let visited_urls = visited_urls.clone();
                    async move {
                        let mut visited = visited_urls.lock().await;
                        if visited.contains(&link) {
                            return Ok(vec![]);
                        }
                        visited.insert(link.clone());
                        drop(visited);

                        self.fetch_page_with_client(&link, &client).await
                    }
                })
                .buffer_unordered(self.config.concurrency)
                .collect()
                .await;

            for result in results {
                if let Ok(mut sub_pages) = result {
                    pages.append(&mut sub_pages);
                }
            }
        }

        Ok(pages)
    }

    async fn fetch_page(&self, url: &str) -> Result<DocPage> {
        self.fetch_page_with_client(url, &self.client).await
    }

    async fn fetch_page_with_client(&self, url: &str, client: &Client) -> Result<DocPage> {
        let mut backoff = ExponentialBackoff::default();
        backoff.max_elapsed_time = Some(self.config.timeout);
        
        let response = backoff::future::retry(backoff, || async {
            Ok(client.get(url).send().await?)
        }).await?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);
        
        // 解析页面内容
        let title_selector = Selector::parse("h1").unwrap();
        let content_selector = Selector::parse("article").unwrap();
        let links_selector = Selector::parse("a[href]").unwrap();

        let title = document
            .select(&title_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let content = document
            .select(&content_selector)
            .next()
            .map(|el| el.text().collect::<String>())
            .unwrap_or_default();

        let base_url = Url::parse(url)?;
        let related_links: Vec<String> = document
            .select(&links_selector)
            .filter_map(|el| {
                el.value().attr("href").and_then(|href| {
                    base_url.join(href).ok().map(|url| url.to_string())
                })
            })
            .filter(|url| url.contains("developer.apple.com"))
            .collect();

        Ok(DocPage {
            title,
            content,
            url: url.to_string(),
            related_links,
        })
    }

    pub async fn search_and_crawl(&mut self, keyword: &str, recursive: bool) -> Result<Vec<DocPage>> {
        let search_url = format!(
            "https://developer.apple.com/search/index.php?q={}",
            urlencoding::encode(keyword)
        );
        self.crawl_url(&search_url, recursive).await
    }
} 
