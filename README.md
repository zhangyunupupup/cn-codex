<div align="center">

# 🇨🇳 cn-codex

让 OpenAI Codex CLI 开箱即用国产大模型

[![CI](https://img.shields.io/github/actions/workflow/status/zhangyunupupup/cn-codex/ci.yml?branch=main&label=CI&logo=github)](https://github.com/zhangyunupupup/cn-codex/actions)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.80%2B-orange?logo=rust)](https://www.rust-lang.org)
[![Version](https://img.shields.io/github/v/release/zhangyunupupup/cn-codex?label=version)](https://github.com/zhangyunupupup/cn-codex/releases)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

</div>

## 📋 目录

- [为什么需要 cn-codex？](#-为什么需要-cn-codex)
- [快速开始](#-快速开始)
- [支持的模型](#-支持的模型)
- [工作原理](#-工作原理)
- [手动配置](#-手动配置)
- [从源码编译](#-从源码编译)
- [常见问题](#-常见问题)
- [项目结构](#-项目结构)
- [贡献](#-贡献)
- [许可证](#-许可证)

---

## ❓ 为什么需要 cn-codex？

Codex CLI 最新版（v0.123+）**仅支持 OpenAI Responses API**，而国产大模型（DeepSeek、Qwen、GLM 等）普遍只提供 Chat Completions API。直接配置 `wire_api = "chat"` 已被硬删除：

```
`wire_api = "chat"` is no longer supported.
```

cn-codex 在本地启动一个协议桥接代理，**自动将 Responses API 请求翻译为 Chat Completions 请求**，对 Codex 完全透明，无需任何代理网关或 Docker。

---

## 🚀 快速开始

### 前置条件

| 依赖 | 说明 | 安装方式 |
|------|------|---------|
| **Codex CLI** | OpenAI 编程代理 | `npm install -g @openai/codex` 或 `brew install codex` |
| **Node.js / npm** | 安装 Codex CLI | 从 [nodejs.org](https://nodejs.org/) 下载 |

> **⛔ Windows 用户请注意：不要双击 .sh 文件！**
> 正确操作：打开 **Git Bash**（或 **WSL**），在终端中输入：
> ```bash
> cd 下载目录
> bash install.sh
> ```

### 安装

```bash
# 方式一：一键安装（推荐）
curl -fsSL https://raw.githubusercontent.com/zhangyunupupup/cn-codex/main/install.sh | bash

# 方式二：克隆后安装
git clone https://github.com/zhangyunupupup/cn-codex.git
cd cn-codex
./install.sh
```

### 配置

```bash
cn-codex-setup
```

交互式选择模型提供商 → 输入 API Key → 完成。

### 使用

```bash
# 直接使用
cn-codex "帮我写个 Python 快速排序"

# 管理桥接代理
cn-codex bridge status    # 查看状态
cn-codex bridge restart  # 重启代理
cn-codex bridge log      # 查看日志

# 先启动桥接代理，再单独使用 codex 命令
codex "继续未完成的任务"
```

---

## 🧠 支持的模型

| 提供商 | Provider ID | 默认模型 | API Key 获取 |
|--------|-----------|---------|-------------|
| **DeepSeek** | `deepseek` | deepseek-chat | [获取 Key](https://platform.deepseek.com/api_keys) |
| **通义千问 (Qwen)** | `qwen` | qwen-plus | [获取 Key](https://dashscope.console.aliyun.com/apiKey) |
| **智谱 GLM** | `zhipu` | glm-4-plus | [获取 Key](https://open.bigmodel.cn/usercenter/apikeys) |
| **Kimi (月之暗面)** | `kimi` | moonshot-v1-8k | [获取 Key](https://platform.moonshot.cn/console/api-keys) |
| **豆包 (字节跳动)** | `doubao` | doubao-pro-32k | [获取 Key](https://console.volcengine.com/ark/region:ark+cn-beijing/apiKey) |
| **SiliconFlow** | `siliconflow` | DeepSeek-V3 | [获取 Key](https://cloud.siliconflow.cn/account/ak) |
| 自定义 API | — | — | 你自己的 API |

---

## 🔧 工作原理

```
┌─────────────┐    Responses API    ┌──────────────────┐   Chat Completions   ┌──────────────┐
│  Codex CLI   │ ──── /v1/responses ──►  cn-codex-bridge  │ ── /v1/chat/completions ──►  国产大模型 API  │
│  (终端)      │ ◄── SSE events ─────  (本地代理)         │ ◄── SSE chunks ──────────  (DeepSeek等)  │
└─────────────┘                      └──────────────────┘                       └──────────────┘
     ↕ 127.0.0.1:15721                       ↕ 协议转换
```

1. **cn-codex** 启动本地桥接代理（默认 `127.0.0.1:15721`）
2. 自动修改 `~/.codex/config.toml`，将 `base_url` 指向本地代理
3. Codex CLI 按 Responses API 格式发送请求到本地代理
4. 代理将请求转换为 Chat Completions 格式，转发给国产模型
5. 代理将 Chat Completions 响应转换回 Responses API 格式，返回给 Codex

### 协议转换细节

| 方向 | 转换内容 |
|------|---------|
| 请求 | `input[]` → `messages[]`，`instructions` → `system` message，`function` tools → Chat tools |
| 响应 | `choices[].message` → `output[]` items，`tool_calls` → `function_call` items |
| 流式 | Chat SSE `delta.content` → Responses `output_text.delta`，`delta.tool_calls` → `function_call_arguments.delta` |

---

## ⚙️ 手动配置

如果你不想用 `cn-codex-setup`，可以手动配置：

### 1. 设置环境变量

```bash
export DEEPSEEK_API_KEY="sk-your-key"
```

### 2. 编辑 `~/.codex/config.toml`

```toml
model = "deepseek-chat"
model_provider = "cn-bridge"

[model_providers.cn-bridge]
name = "CN Bridge (DeepSeek)"
base_url = "http://127.0.0.1:15721/v1"
wire_api = "responses"
stream_idle_timeout_ms = 600000
```

### 3. 启动桥接代理

```bash
cn-codex-bridge \
  --port 15721 \
  --upstream-url https://api.deepseek.com/v1 \
  --api-key-env DEEPSEEK_API_KEY
```

### 4. 运行 Codex

```bash
codex "你的任务"
```

---

## 🏗️ 从源码编译

```bash
# 需要 Rust 工具链 (https://rustup.rs)
git clone https://github.com/zhangyunupupup/cn-codex.git
cd cn-codex/bridge
cargo build --release

# 二进制位于 target/release/cn-codex-bridge (约 4.9MB)
cp target/release/cn-codex-bridge ~/.cn-codex/bin/
```

---

## ❔ 常见问题

<details>
<summary><b>为什么不用 wire_api = "chat" 直接配？</b></summary>

Codex CLI v0.123+ 已硬删除 `wire_api = "chat"` 支持（[PR #10157](https://github.com/openai/codex/pull/10157)），配置后会报错：
```
`wire_api = "chat"` is no longer supported.
```
cn-codex 通过本地协议桥接绕过了这个限制。
</details>

<details>
<summary><b>和 Codex++ 有什么区别？</b></summary>

| | cn-codex | Codex++ |
|---|---|---|
| 目标 | Codex CLI（终端） | Codex App（桌面） |
| 原理 | 本地协议桥接代理 | CDP 注入 App |
| 依赖 | 无额外依赖 | Tauri + Rust |
| 开源 | ✅ | ✅ |
</details>

<details>
<summary><b>桥接代理会影响性能吗？</b></summary>

代理运行在本地 `127.0.0.1`，延迟极低（<1ms）。主要开销是 JSON 序列化/反序列化，对实际使用几乎无感知。
</details>

<details>
<summary><b>支持工具调用（function calling）吗？</b></summary>

✅ 支持。桥接代理会自动将 Responses API 的 function_call 转换为 Chat Completions 的 tool_calls，反之亦然。
</details>

<details>
<summary><b>支持 DeepSeek R1 推理模式吗？</b></summary>

✅ 支持。DeepSeek R1 返回的 `reasoning_content` 会被转换为 Responses API 的 `reasoning` 事件。
</details>

更多问题请查看 [FAQ](docs/faq.md)。

---

## 📁 项目结构

```
cn-codex/
├── cn-codex              # 主入口 wrapper (Bash)
├── install.sh            # 一键安装脚本
├── bin/
│   └── cn-codex-setup    # 交互式配置工具 (Bash)
├── bridge/               # 协议桥接代理 (Rust)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs       # 入口
│       ├── config.rs     # 配置
│       ├── converter.rs  # 协议转换核心 (含4个单元测试)
│       ├── proxy.rs      # HTTP 代理层
│       └── server.rs     # HTTP 服务器
├── providers/            # 预置国产模型配置 (TOML)
│   ├── deepseek.toml
│   ├── qwen.toml
│   ├── zhipu.toml
│   ├── kimi.toml
│   ├── doubao.toml
│   └── siliconflow.toml
├── docs/
│   └── faq.md
├── .github/
│   └── workflows/
│       └── ci.yml        # CI/CD: 编译+测试+跨平台发布
├── CONTRIBUTING.md        # 贡献指南
├── CHANGELOG.md           # 变更日志
└── LICENSE                # Apache-2.0
```

---

## 🤝 贡献

欢迎贡献！详见 [CONTRIBUTING.md](CONTRIBUTING.md)。特别需要：

- 更多国产模型 provider 配置
- 桥接代理功能增强（更多 Responses API 特性支持）
- 文档和测试
- Windows 适配

---

## 📄 许可证

[Apache-2.0](LICENSE) — 详见项目 LICENSE 文件。

---

## 🙏 致谢

- [OpenAI Codex CLI](https://github.com/openai/codex) — 上游项目
- [va-ai-api-bridge](https://github.com/jazzenchen/va-ai-api-bridge) — 协议转换设计参考
- [CC Switch](https://github.com/farion1231/cc-switch) — 路由方案参考
