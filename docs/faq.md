# 常见问题

## 通用

### Q: 为什么不用 `wire_api = "chat"` 直接配置？

Codex CLI v0.123+ 已硬删除 `wire_api = "chat"` 支持（[PR #10157](https://github.com/openai/codex/pull/10157)）。如果配置 `wire_api = "chat"`，会收到以下错误：

```
`wire_api = "chat"` is no longer supported.
How to fix: set `wire_api = "responses"` in your provider config.
```

cn-codex 通过本地协议桥接代理绕过了这个限制——Codex 以为自己连的是 Responses API，实际上请求被翻译成了 Chat Completions API。

### Q: cn-codex 和其他方案有什么区别？

| 方案 | 原理 | 优点 | 缺点 |
|------|------|------|------|
| **cn-codex** | 本地协议桥接代理 | 零外部依赖，自动配置 | 需运行代理进程 |
| Codex++ | CDP 注入桌面版 App | GUI 友好 | 仅限桌面版 |
| AxonHub/CCX | Docker 网关 | 功能全面 | 需 Docker，重 |
| CC Switch | Node 代理路由 | 灵活 | 需额外进程 |
| 手动降级 Codex | 用旧版 CLI | 最简单 | 缺少新功能 |

### Q: 桥接代理会影响性能吗？

代理运行在本地 `127.0.0.1`，延迟极低（<1ms）。主要开销是 JSON 序列化/反序列化，对实际使用几乎无感知。

### Q: 安全吗？API Key 会不会泄露？

- API Key 存储在本地文件 `~/.cn-codex/api-keys.env`（权限 600）
- 桥接代理仅监听 `127.0.0.1`，外部无法访问
- API Key 不会出现在 Codex CLI 的配置中

## 模型相关

### Q: 推荐哪个模型？

| 用途 | 推荐模型 | 理由 |
|------|---------|------|
| 日常编程 | DeepSeek Chat | 性价比最高，编程能力强 |
| 复杂推理 | DeepSeek Reasoner | R1 推理模型，逻辑强 |
| 快速响应 | Qwen Turbo | 速度快，延迟低 |
| 长上下文 | Moonshot V1 128K | 支持超长代码 |
| 多模型切换 | SiliconFlow | 一个 Key 用多个模型 |

### Q: 支持 DeepSeek R1 推理模式吗？

✅ 支持。桥接代理会将 DeepSeek R1 返回的 `reasoning_content` 转换为 Responses API 的 `reasoning` 事件。

### Q: 支持工具调用（function calling）吗？

✅ 支持。桥接代理会自动将 Responses API 的 `function_call` 转换为 Chat Completions 的 `tool_calls`，反之亦然。

### Q: 为什么我的模型报 "Incorrect role information" 错误？

部分国产模型对消息格式有严格要求。常见原因：
- 系统消息格式不对 → 确保使用 `instructions` 字段
- 连续的 assistant 消息 → 检查对话历史是否正确
- 工具调用格式不对 → 确保模型支持 function calling

## 故障排除

### Q: 桥接代理启动失败

1. 检查端口是否被占用：`lsof -i :15721`
2. 查看日志：`cn-codex bridge log`
3. 尝试换个端口：`export CN_CODEX_BRIDGE_PORT=15722`

### Q: Codex CLI 连接超时

1. 确认桥接代理正在运行：`cn-codex bridge status`
2. 测试代理连通性：`curl http://127.0.0.1:15721/health`
3. 检查 `~/.codex/config.toml` 中 `base_url` 是否正确
4. 增加超时：在 config.toml 中设置 `stream_idle_timeout_ms = 600000`

### Q: 请求报 401 Unauthorized

1. 确认 API Key 环境变量已设置：`echo $DEEPSEEK_API_KEY`
2. 重新运行 `cn-codex-setup`
3. 检查 `source ~/.cn-codex/env.sh` 是否已添加到 shell 配置

### Q: 流式响应断开

1. 国产模型推理速度可能较慢，增大超时时间
2. 在 `~/.codex/config.toml` 中设置 `stream_idle_timeout_ms = 600000`
3. 某些模型（如 DeepSeek Reasoner）推理时间较长，属于正常现象
