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
use tracing::{info, warn, debug, error, instrument};
use indicatif::{ProgressBar, ProgressStyle};

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
    fn clean_text(text: &str) -> String {
        text.lines()  // 按行分割
            .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))  // 清理每行的空白
            .filter(|line| !line.is_empty())  // 移除空行
            .collect::<Vec<_>>()  // 收集到 Vec
            .join("\n")  // 用换行符重新连接
            .trim()  // 去除首尾空白
            .to_string()
    }

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

    #[instrument(skip(self))]
    pub async fn crawl_url(&mut self, url: &str, recursive: bool) -> Result<Vec<DocPage>> {
        let mut pages = Vec::new();
        
        // 创建主进度条
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap()
        );
        spinner.set_message(format!("爬取页面: {}", url));
        
        {
            let mut visited = self.visited_urls.lock().await;
            if visited.contains(url) {
                debug!("跳过已访问的 URL: {}", url);
                return Ok(pages);
            }
            visited.insert(url.to_string());
            debug!("添加 URL 到已访问列表: {}", url);
        }
        
        info!("开始爬取页面: {}", url);
        let start = std::time::Instant::now();
        match self.fetch_page(url).await {
            Ok(page) => {
                let elapsed = start.elapsed();
                info!(
                    "成功爬取页面: {}, 耗时: {:.2}s, 标题: {}", 
                    url, 
                    elapsed.as_secs_f64(),
                    page.title
                );
                pages.push(page);
            }
            Err(e) => {
                error!("爬取页面失败: {}, 错误: {}", url, e);
                return Err(e);
            }
        }

        if recursive {
            let links: Vec<String> = pages[0].related_links.clone();
            info!("发现 {} 个相关链接，开始并发爬取", links.len());
            
            // 创建多进度条
            let progress = ProgressBar::new(links.len() as u64);
            progress.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                    .unwrap()
                    .progress_chars("#>-")
            );
            
            let client = self.client.clone();
            let visited_urls = self.visited_urls.clone();
            let config = &self.config;
            
            let results: Vec<_> = stream::iter(links)
                .map(|link| {
                    let client = client.clone();
                    let visited_urls = visited_urls.clone();
                    let progress = progress.clone();
                    async move {
                        let mut visited = visited_urls.lock().await;
                        if visited.contains(&link) {
                            progress.inc(1);
                            progress.set_message(format!("跳过: {}", link));
                            return Ok::<Vec<DocPage>, anyhow::Error>(vec![]);
                        }
                        visited.insert(link.clone());
                        progress.set_message(format!("爬取: {}", link));
                        drop(visited);

                        match self.fetch_page_with_client(&link, &client).await {
                            Ok(page) => {
                                progress.inc(1);
                                progress.set_message(format!("成功: {}", link));
                                Ok(vec![page])
                            }
                            Err(e) => {
                                progress.inc(1);
                                progress.set_message(format!("失败: {}", link));
                                warn!("爬取相关页面失败: {}, 错误: {}", link, e);
                                Ok(vec![])
                            }
                        }
                    }
                })
                .buffer_unordered(config.concurrency)
                .collect()
                .await;

            let mut success_count = 0;
            for result in results {
                if let Ok(mut sub_pages) = result {
                    success_count += sub_pages.len();
                    pages.append(&mut sub_pages);
                }
            }
            
            progress.finish_with_message(format!("完成！成功爬取 {} 个页面", success_count));
        }

        spinner.finish_with_message(format!("完成！共获取 {} 个页面", pages.len()));
        Ok(pages)
    }

    pub async fn search_and_crawl(&mut self, keyword: &str, recursive: bool) -> Result<Vec<DocPage>> {
        let search_url = format!(
            "https://developer.apple.com/search/index.php?q={}",
            urlencoding::encode(keyword)
        );
        self.crawl_url(&search_url, recursive).await
    }

    async fn fetch_page(&mut self, url: &str) -> Result<DocPage> {
        let mut backoff = ExponentialBackoff::default();
        backoff.max_elapsed_time = Some(self.config.timeout);
        
        let start = std::time::Instant::now();
        debug!("开始请求页面: {}", url);
        
        let response = backoff::future::retry(backoff, || async {
            let request_start = std::time::Instant::now();
            match self.client.get(url).send().await {
                Ok(resp) => {
                    let elapsed = request_start.elapsed();
                    debug!(
                        "请求成功: {}, 状态码: {}, 耗时: {:.2}s",
                        url,
                        resp.status(),
                        elapsed.as_secs_f64()
                    );
                    Ok(resp)
                }
                Err(e) => {
                    warn!("请求失败，准备重试: {}, 错误: {}", url, e);
                    Err(e.into())
                }
            }
        }).await?;

        let html = response.text().await?;
        let document = Html::parse_document(&html);
        
        let title_selector = Selector::parse("h1").unwrap();
        let content_selector = Selector::parse("article").unwrap();
        let links_selector = Selector::parse("a[href]").unwrap();

        let title = document
            .select(&title_selector)
            .next()
            .map(|el| Self::clean_text(&el.text().collect::<String>()))
            .unwrap_or_default();

        let content = document
            .select(&content_selector)
            .next()
            .map(|el| Self::clean_text(&el.text().collect::<String>()))
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

        let elapsed = start.elapsed();
        info!(
            "页面处理完成: {}, 总耗时: {:.2}s",
            url,
            elapsed.as_secs_f64()
        );
        
        Ok(DocPage {
            title,
            content,
            url: url.to_string(),
            related_links,
        })
    }
} 
