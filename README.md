<div align="center">

# 🍎 ADOC

*一个快速、高效的 Apple 开发者文档爬虫工具*

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](Cargo.toml)

</div>

## ✨ 特性

- 🚀 **异步并发**: 利用 Rust 异步特性，支持高并发爬取
- 🔄 **递归爬取**: 可选择递归爬取相关文档页面
- 📦 **多种输出**: 支持 JSON、美化 JSON 和文本格式
- 🛠 **可配置**: 灵活的命令行参数配置
- 🔍 **智能搜索**: 支持关键词搜索和直接 URL 爬取
- 🔄 **自动重试**: 内置智能重试机制

## 🚀 安装 

```bash
gcl git@github.com:king-open/adoc.git 

cd adoc
```

* 编译安装

```bash
cargo install --path . 
```


## 📖 使用方法

### 基本用法


```bash
# 搜索 SwiftUI 文档

adoc -i "SwiftUI" -o swiftui.json

# 递归爬取 UIKit 文档

adoc -i "UIKit" -r -o uikit.txt

# 使用 10 个并发任务爬取

adoc -i https://developer.apple.com/documentation/swift -c 10
```


## 🛠 技术栈

- **异步运行时**: [tokio](https://tokio.rs/)
- **HTTP 客户端**: [reqwest](https://docs.rs/reqwest)
- **命令行解析**: [clap](https://docs.rs/clap)
- **HTML 解析**: [scraper](https://docs.rs/scraper)
- **错误处理**: [anyhow](https://docs.rs/anyhow)
- **并发控制**: [futures](https://docs.rs/futures)

## 📝 TODO

- [ ] 添加进度条显示
- [ ] 支持导出为 Markdown 格式
- [ ] 添加代理支持
- [ ] 实现断点续传
- [ ] 添加测试用例

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情





