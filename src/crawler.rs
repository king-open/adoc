use anyhow::Result;
use backoff::ExponentialBackoff;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashSet;
use url::Url;

#[derive(Debug, Serialize)]
pub struct DocPage {
    pub title: String,
    pub content: String,
    pub url: String,
    pub related_links: Vec<String>,
}

pub struct Crawler {
    client: Client,
    max_retries: u32,
    visited_urls: HashSet<String>,
}

impl Crawler {
    pub fn new(max_retries: u32) -> Self {
        Self {
            client: Client::new(),
            max_retries,
            visited_urls: HashSet::new(),
        }
    }

    pub async fn crawl_url(&mut self, url: &str, recursive: bool) -> Result<Vec<DocPage>> {
        self.crawl_url_inner(url, recursive).await
    }

    async fn crawl_url_inner(&mut self, url: &str, recursive: bool) -> Result<Vec<DocPage>> {
        let mut pages = Vec::new();
        
        if self.visited_urls.contains(url) {
            return Ok(pages);
        }
        
        let page = self.fetch_page(url).await?;
        self.visited_urls.insert(url.to_string());
        pages.push(page);

        if recursive {
            let links: Vec<String> = pages[0].related_links.clone();
            for link in links {
                if !self.visited_urls.contains(&link) {
                    let future = Box::pin(self.crawl_url_inner(&link, true));
                    let mut sub_pages = future.await?;
                    pages.append(&mut sub_pages);
                }
            }
        }

        Ok(pages)
    }

    async fn fetch_page(&self, url: &str) -> Result<DocPage> {
        let mut backoff = ExponentialBackoff::default();
        backoff.max_elapsed_time = Some(std::time::Duration::from_secs(
            self.max_retries as u64 * 10
        ));
        
        let response = backoff::future::retry(backoff, || async {
            Ok(self.client.get(url).send().await?)
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
