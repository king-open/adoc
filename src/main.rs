mod crawler;
mod output;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use crate::crawler::{Crawler, CrawlerConfig};
use crate::output::{save_results, print_results, OutputFormat};
use tracing::{info};

fn setup_logging(level: &str) {
    use tracing_subscriber::{fmt, EnvFilter};
    use time::macros::format_description;

    let timer_format = format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
    
    fmt()
        .with_env_filter(EnvFilter::new(level))
        .with_timer(fmt::time::UtcTime::new(timer_format))
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true)
        .init();
}

#[derive(Parser, Debug)]
#[command(
    name = "adoc",
    author = "JackW",
    version,
    about = "Apple 开发者文档爬虫工具 (Apple Documentation Crawler)",
    long_about = "一个快速、高效的 Apple 开发者文档爬虫工具。

示例:
    adoc -i \"SwiftUI\" -o swiftui.json       # 搜索 SwiftUI 文档并保存为 JSON
    adoc -i \"UIKit\" -r -o uikit.txt         # 递归爬取 UIKit 相关文档
    adoc -i https://developer.apple.com/documentation/swift -c 10  # 使用10个并发任务爬取"
)]
struct Args {
    /// Apple 开发者文档 URL 或关键字
    /// 例如: https://developer.apple.com/documentation/swift 或 "SwiftUI"
    #[arg(short, long, help_heading = "输入选项")]
    input: String,

    /// 是否递归爬取相关页面
    /// 启用此选项将爬取文档中引用的其他页面
    #[arg(short, long, default_value = "false", help_heading = "爬取选项")]
    recursive: bool,

    /// 并发任务数
    /// 控制同时进行的爬取任务数量
    #[arg(short = 'c', long, default_value = "5", help_heading = "爬取选项")]
    concurrency: usize,

    /// 输出文件路径
    /// 支持 .json 或 .txt 格式，例如: output.json 或 docs.txt
    #[arg(short, long, help_heading = "输出选项")]
    output: Option<PathBuf>,

    /// 输出格式
    /// 可选: json, txt, pretty_json
    #[arg(short = 'f', long, default_value = "json", help_heading = "输出选项")]
    format: OutputFormat,

    /// 网络请求最大重试次数
    #[arg(short, long, default_value = "3", help_heading = "网络选项")]
    max_retries: u32,

    /// 请求超时时间（秒）
    #[arg(short = 't', long, default_value = "30", help_heading = "网络选项")]
    timeout: u64,

    /// 日志级别
    /// 可选: error, warn, info, debug, trace
    #[arg(short = 'l', long, default_value = "info", help_heading = "日志选项")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // 设置日志
    setup_logging(&args.log_level);
    info!("启动 adoc 爬虫工具...");
    
    let config = CrawlerConfig {
        max_retries: args.max_retries,
        concurrency: args.concurrency,
        timeout: std::time::Duration::from_secs(args.timeout),
    };
    
    info!(
        "配置信息: 并发数={}, 超时={}s, 重试次数={}", 
        config.concurrency, 
        config.timeout.as_secs(), 
        config.max_retries
    );
    
    let mut crawler = Crawler::new(config);
    
    info!("开始爬取: {}", args.input);
    let results = if args.input.starts_with("http") {
        crawler.crawl_url(&args.input, args.recursive).await?
    } else {
        crawler.search_and_crawl(&args.input, args.recursive).await?
    };
    info!("爬取完成，共获取 {} 个页面", results.len());

    if let Some(output_path) = args.output {
        info!("保存结果到文件: {}", output_path.display());
        save_results(&results, &output_path, args.format)?;
        info!("文件保存成功");
    } else {
        info!("打印结果到控制台");
        print_results(&results, args.format);
    }

    info!("任务完成");
    Ok(())
}
