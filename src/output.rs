use anyhow::Result;
use clap::{ValueEnum, Parser};
use std::path::Path;
use crate::crawler::DocPage;

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    #[value(name = "pretty")]
    PrettyJson,
    Txt,
    Markdown,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::PrettyJson => write!(f, "pretty"),
            OutputFormat::Txt => write!(f, "txt"),
            OutputFormat::Markdown => write!(f, "markdown"),
        }
    }
}

pub fn save_results(results: &[DocPage], output_path: &Path, format: OutputFormat) -> Result<()> {
    let content = match format {
        OutputFormat::Json => serde_json::to_string(results)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(results)?,
        OutputFormat::Txt => format_as_text(results),
        OutputFormat::Markdown => format_as_markdown(results),
    };

    std::fs::write(output_path, content)?;
    Ok(())
}

pub fn print_results(results: &[DocPage], format: OutputFormat) {
    let content = match format {
        OutputFormat::Json => serde_json::to_string(results).unwrap(),
        OutputFormat::PrettyJson => serde_json::to_string_pretty(results).unwrap(),
        OutputFormat::Txt => format_as_text(results),
        OutputFormat::Markdown => format_as_markdown(results),
    };
    println!("{}", content);
}

fn format_as_text(results: &[DocPage]) -> String {
    let mut content = String::new();
    for page in results {
        content.push_str(&format!("标题: {}\n", page.title));
        content.push_str(&format!("URL: {}\n", page.url));
        content.push_str(&format!("内容:\n{}\n", page.content));
        content.push_str("\n---\n\n");
    }
    content
}

fn format_as_markdown(results: &[DocPage]) -> String {
    let mut content = String::new();
    
    // 添加文档标题
    content.push_str("# Apple 开发者文档\n\n");
    content.push_str("*由 adoc 工具爬取的文档内容*\n\n");
    
    // 添加目录
    content.push_str("## 目录\n\n");
    for (i, page) in results.iter().enumerate() {
        content.push_str(&format!("{}. [{}](#doc-{})\n", i + 1, page.title, i + 1));
    }
    content.push_str("\n---\n\n");

    // 添加每个文档的详细内容
    for (i, page) in results.iter().enumerate() {
        // 文档标题和链接
        content.push_str(&format!("## <a id=\"doc-{}\">{}</a>\n\n", i + 1, page.title));
        content.push_str(&format!("> 原始链接: [{}]({})\n\n", page.url, page.url));
        
        // 文档内容
        content.push_str("### 内容\n\n");
        // 将内容按段落分割并格式化
        for paragraph in page.content.split("\n\n") {
            if !paragraph.trim().is_empty() {
                content.push_str(&format!("{}\n\n", paragraph.trim()));
            }
        }
        
        // 相关链接
        if !page.related_links.is_empty() {
            content.push_str("### 相关链接\n\n");
            for link in &page.related_links {
                content.push_str(&format!("- [{}]({})\n", link, link));
            }
            content.push_str("\n");
        }
        
        content.push_str("---\n\n");
    }

    // 添加页脚
    content.push_str("## 关于\n\n");
    content.push_str("本文档由 [adoc](https://github.com/king-open/adoc) 自动生成。\n");
    
    content
} 
