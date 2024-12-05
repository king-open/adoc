use anyhow::Result;
use clap::ValueEnum;
use std::path::Path;
use crate::crawler::DocPage;

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Json,
    PrettyJson,
    Txt,
}

pub fn save_results(results: &[DocPage], output_path: &Path, format: OutputFormat) -> Result<()> {
    let content = match format {
        OutputFormat::Json => serde_json::to_string(results)?,
        OutputFormat::PrettyJson => serde_json::to_string_pretty(results)?,
        OutputFormat::Txt => {
            let mut content = String::new();
            for page in results {
                content.push_str(&format!("标题: {}\n", page.title));
                content.push_str(&format!("URL: {}\n", page.url));
                content.push_str(&format!("内容:\n{}\n", page.content));
                content.push_str("\n---\n\n");
            }
            content
        }
    };

    std::fs::write(output_path, content)?;
    Ok(())
}

pub fn print_results(results: &[DocPage], format: OutputFormat) {
    match format {
        OutputFormat::Json => println!("{}", serde_json::to_string(results).unwrap()),
        OutputFormat::PrettyJson => println!("{}", serde_json::to_string_pretty(results).unwrap()),
        OutputFormat::Txt => {
            for page in results {
                println!("标题: {}", page.title);
                println!("URL: {}", page.url);
                println!("内容预览: {:.200}...", page.content);
                println!("相关链接数量: {}", page.related_links.len());
                println!("---\n");
            }
        }
    }
} 
