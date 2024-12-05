use anyhow::Result;
use std::path::Path;
use crate::crawler::DocPage;

pub fn save_results(results: &[DocPage], output_path: &Path) -> Result<()> {
    let extension = output_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    match extension {
        "json" => {
            let json = serde_json::to_string_pretty(results)?;
            std::fs::write(output_path, json)?;
        }
        "txt" => {
            let mut content = String::new();
            for page in results {
                content.push_str(&format!("标题: {}\n", page.title));
                content.push_str(&format!("URL: {}\n", page.url));
                content.push_str(&format!("内容:\n{}\n", page.content));
                content.push_str("\n---\n\n");
            }
            std::fs::write(output_path, content)?;
        }
        _ => anyhow::bail!("不支持的输出文件格式：{}", extension),
    }

    Ok(())
}

pub fn print_results(results: &[DocPage]) {
    for page in results {
        println!("标题: {}", page.title);
        println!("URL: {}", page.url);
        println!("内容预览: {:.200}...", page.content);
        println!("相关链接数量: {}", page.related_links.len());
        println!("---\n");
    }
} 
