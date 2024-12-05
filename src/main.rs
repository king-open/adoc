mod crawler;
mod output;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use crate::crawler::Crawler;
use crate::output::{save_results, print_results};

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
    adoc -i https://developer.apple.com/documentation/swift  # 直接爬取 Swift 文档"
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

    /// 输出文件路径
    /// 支持 .json 或 .txt 格式，例如: output.json 或 docs.txt
    #[arg(short, long, help_heading = "输出选项")]
    output: Option<PathBuf>,

    /// 网络请求最大重试次数
    #[arg(short, long, default_value = "3", help_heading = "网络选项")]
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
