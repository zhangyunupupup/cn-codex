# 变更日志

## v0.1.0 (未发布)

### ✨ 新增

- **协议桥接代理** (`cn-codex-bridge`): 用 Rust 实现的本地代理，将 OpenAI Responses API 请求翻译为 Chat Completions 协议 (DeepSeek、Qwen、GLM、Kimi 等国产模型)
- **一键安装脚本** (`install.sh`): 自动检测平台、安装 Codex CLI、下载/编译桥接代理、配置 PATH
- **交互式配置工具** (`cn-codex-setup`): 引导式选择模型提供商、输入 API Key、自动生成 `~/.codex/config.toml`
- **主入口 wrapper** (`cn-codex`): 自动启动桥接代理 + 透传参数给 Codex CLI
- **预置 6 套国产模型配置**: DeepSeek、Qwen、GLM、Kimi、豆包、SiliconFlow
- **跨平台 CI/CD**: GitHub Actions 自动编译 Linux/macOS/Windows 二进制
- **协议转换**: 支持 `input[]` ↔ `messages[]`、`function_call` ↔ `tool_calls`、SSE 流式事件转换
- **中文文档**: 完整的中文 README、FAQ、贡献指南

### 🧪 测试

- 4 个 Rust 单元测试覆盖核心协议转换逻辑

### 🏗️ 项目结构

```
cn-codex/
├── cn-codex              # Bash wrapper
├── install.sh            # 安装脚本
├── bin/
│   └── cn-codex-setup    # 配置工具
├── bridge/               # Rust 协议桥接代理
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs       # CLI 入口
│       ├── config.rs     # 配置
│       ├── converter.rs  # 协议转换核心
│       ├── proxy.rs      # HTTP 代理层
│       └── server.rs     # HTTP 服务器 (axum)
├── providers/            # 6 套国产模型配置
├── docs/
│   └── faq.md
└── .github/workflows/
    └── ci.yml            # CI/CD
```
