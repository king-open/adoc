use adoc::crawler::{Crawler, CrawlerConfig, DocPage};
use std::time::Duration;

#[tokio::test]
async fn test_crawler_with_logging() {
    // 设置测试日志
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .init();

    // 创建爬虫配置
    let config = CrawlerConfig {
        max_retries: 3,
        concurrency: 2,
        timeout: Duration::from_secs(30),
    };

    let mut crawler = Crawler::new(config);

    // 测试单个页面爬取
    let results = crawler
        .crawl_url("https://developer.apple.com/documentation/swift", false)
        .await
        .unwrap();

    assert!(!results.is_empty());
    assert!(results[0].title.contains("Swift"));

    // 测试递归爬取（限制深度）
    let results = crawler
        .crawl_url("https://developer.apple.com/documentation/swiftui", true)
        .await
        .unwrap();

    assert!(results.len() > 1);
}

#[tokio::test]
async fn test_search_and_crawl() {
    // 测试搜索功能
    let config = CrawlerConfig {
        max_retries: 3,
        concurrency: 2,
        timeout: Duration::from_secs(30),
    };

    let mut crawler = Crawler::new(config);
    let results = crawler.search_and_crawl("SwiftUI", false).await.unwrap();

    assert!(!results.is_empty());
} 
