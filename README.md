# 🇨🇳 cn-codex

> 让 OpenAI Codex CLI 开箱即用国产大模型

cn-codex 是一个轻量工具，让 [OpenAI Codex CLI](https://github.com/openai/codex) 无缝接入 DeepSeek、通义千问、智谱 GLM、Kimi 等国产大模型。

## 为什么需要 cn-codex？

Codex CLI 最新版（v0.123+）**仅支持 OpenAI Responses API**，而国产大模型普遍只提供 Chat Completions API。直接配置 `wire_api = "chat"` 已被硬删除，会报错。

cn-codex 在本地启动一个协议桥接代理，**自动将 Responses API 请求翻译为 Chat Completions 请求**，对 Codex 完全透明，无需任何代理网关或 Docker。

## 快速开始

### 安装

```bash
# 一键安装
curl -fsSL https://raw.githubusercontent.com/用户名/cn-codex/main/install.sh | bash

# 或手动安装
git clone https://github.com/用户名/cn-codex.git
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
```

## 支持的模型

| 提供商 | 默认模型 | API Key 获取 |
|--------|---------|-------------|
| **DeepSeek** | deepseek-chat | [获取 Key](https://platform.deepseek.com/api_keys) |
| **通义千问 (Qwen)** | qwen-plus | [获取 Key](https://dashscope.console.aliyun.com/apiKey) |
| **智谱 GLM** | glm-4-plus | [获取 Key](https://open.bigmodel.cn/usercenter/apikeys) |
| **Kimi (月之暗面)** | moonshot-v1-8k | [获取 Key](https://platform.moonshot.cn/console/api-keys) |
| **豆包 (字节跳动)** | doubao-pro-32k | [获取 Key](https://console.volcengine.com/ark/region:ark+cn-beijing/apiKey) |
| **SiliconFlow** | DeepSeek-V3 | [获取 Key](https://cloud.siliconflow.cn/account/ak) |
| 自定义 API | — | — |

## 工作原理

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
| Request | `input[]` → `messages[]`，`instructions` → `system` message，`function` tools → Chat tools |
| Response | `choices[].message` → `output[]` items，`tool_calls` → `function_call` items |
| Stream | Chat SSE `delta.content` → Responses `output_text.delta`，`delta.tool_calls` → `function_call_arguments.delta` |

## 手动配置

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
cn-codex-bridge --port 15721 --upstream-url https://api.deepseek.com/v1 --api-key-env DEEPSEEK_API_KEY
```

### 4. 运行 Codex

```bash
codex "你的任务"
```

## 从源码编译

```bash
# 需要 Rust 工具链
git clone https://github.com/用户名/cn-codex.git
cd cn-codex/bridge
cargo build --release

# 二进制位于 target/release/cn-codex-bridge
cp target/release/cn-codex-bridge ~/.cn-codex/bin/
```

## 常见问题

<details>
<summary><b>为什么不用 wire_api = "chat" 直接配？</b></summary>

Codex CLI v0.123+ 已硬删除 `wire_api = "chat"` 支持（PR #10157），配置后会报错：
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

## 项目结构

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
│       ├── converter.rs  # 协议转换核心
│       ├── proxy.rs      # HTTP 代理层
│       └── server.rs     # HTTP 服务器
├── providers/            # 预置国产模型配置 (TOML)
│   ├── deepseek.toml
│   ├── qwen.toml
│   ├── zhipu.toml
│   ├── kimi.toml
│   ├── doubao.toml
│   └── siliconflow.toml
└── docs/
    └── faq.md
```

## 贡献

欢迎贡献！特别需要：

- 更多国产模型 provider 配置
- 桥接代理功能增强（更多 Responses API 特性支持）
- 文档和测试
- Windows 适配

## 许可证

Apache-2.0

## 致谢

- [OpenAI Codex CLI](https://github.com/openai/codex) — 上游项目
- [va-ai-api-bridge](https://github.com/jazzenchen/va-ai-api-bridge) — 协议转换设计参考
- [CC Switch](https://github.com/farion1231/cc-switch) — 路由方案参考
