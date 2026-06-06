# 贡献指南

感谢你考虑为 cn-codex 做贡献！无论是报告 bug、提交功能请求、改进文档还是提交代码，我们都欢迎。

## 行为准则

请保持尊重和专业。本项目遵循 [贡献者公约](https://www.contributor-covenant.org/version/2/1/code_of_conduct/)。

## 如何贡献

### 报告 Bug

1. 先搜索 [Issues](https://github.com/zhangyunupupup/cn-codex/issues) 是否已有相同报告
2. 如果没有，[创建新 Issue](https://github.com/zhangyunupupup/cn-codex/issues/new?template=bug_report.md)
3. 清晰描述：
   - 运行环境（操作系统、Shell 类型、Codex CLI 版本）
   - 复现步骤
   - 期望行为和实际行为
   - 相关日志（`cn-codex bridge log` 输出）

### 提交功能请求

[创建功能请求 Issue](https://github.com/zhangyunupupup/cn-codex/issues/new?template=feature_request.md)，说明：

- 你想解决的问题
- 你期望的行为
- 替代方案（如有）

### 提交代码 (Pull Request)

1. **Fork** 本仓库
2. 创建新分支：`git checkout -b feat/xxx`
3. 开发并**确保测试通过**
4. 提交代码：`git commit -m "feat: 简短的描述"`
5. 推送到你的 Fork：`git push origin feat/xxx`
6. 创建 Pull Request

#### Commit 规范

使用 [Conventional Commits](https://www.conventionalcommits.org/)：

| 类型 | 用途 |
|------|------|
| `feat` | 新功能 |
| `fix` | Bug 修复 |
| `docs` | 文档变更 |
| `style` | 代码格式（不影响功能） |
| `refactor` | 重构 |
| `test` | 测试相关 |
| `chore` | 构建/工具变更 |

## 开发环境搭建

### Rust (桥接代理)

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 编译
cd bridge
cargo build

# 运行测试
cargo test

# 格式化和 lint
cargo fmt
cargo clippy
```

### Bash 脚本 (wrapper/installer)

```bash
# 语法检查
bash -n cn-codex
bash -n bin/cn-codex-setup
bash -n install.sh
```

## 代码规范

### Rust

- 运行 `cargo fmt` 确保格式一致
- 无 `cargo clippy` 警告
- 为 `converter.rs` 中的协议转换逻辑添加单元测试
- 使用 `thiserror` 定义错误类型
- 使用 `tracing` 而非 `println` 记录日志

### Bash

- 使用 `#!/usr/bin/env bash`
- 避免 `set -e`，使用手动错误检查
- 使用 `[[ ]]` 而非 `[ ]`
- 变量加 `""` 引号
- 函数命名：snake_case
- 常量命名：UPPER_SNAKE_CASE

## 测试

- Rust 代码使用 `#[cfg(test)]` 单元测试
- 运行 `cargo test` 确保全部通过
- 新增协议转换逻辑必须附带测试用例

## 发布流程

维护者参考：

1. 更新 `CHANGELOG.md`
2. 更新 `bridge/Cargo.toml` 和 `install.sh` 中的版本号
3. 创建 tag：`git tag v0.x.x`
4. 推送 tag：`git push origin v0.x.x`
5. GitHub Actions 自动构建并创建 Release
