mod crawler;
mod output;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use crate::crawler::Crawler;
use crate::output::{save_results, print_results};

#[derive(Parser, Debug)]
#[command(
    author = "JackW",
    version,
    about = "Apple 开发者文档爬虫工具",
    long_about = "一个用于爬取 Apple 开发者文档的命令行工具，支持 URL 直接爬取和关键字搜索。"
)]
struct Args {
    /// Apple 开发者文档 URL 或关键字
    /// 例如: https://developer.apple.com/documentation/swift 或 "SwiftUI"
    #[arg(short, long)]
    input: String,

    /// 是否递归爬取相关页面
    /// 启用此选项将爬取文档中引用的其他页面
    #[arg(short, long, default_value = "false")]
    recursive: bool,

    /// 输出文件路径
    /// 支持 .json 或 .txt 格式，例如: output.json 或 docs.txt
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// 网络请求最大重试次数
    #[arg(short, long, default_value = "3")]
    max_retries: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut crawler = Crawler::new(args.max_retries);
    
    let results = if args.input.starts_with("http") {
        crawler.crawl_url(&args.input, args.recursive).await?
    } else {
        crawler.search_and_crawl(&args.input, args.recursive).await?
    };

    if let Some(output_path) = args.output {
        save_results(&results, &output_path)?;
    } else {
        print_results(&results);
    }

    Ok(())
}
