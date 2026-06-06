#!/usr/bin/env bash
# cn-codex 一键安装脚本
# 用法: curl -fsSL https://raw.githubusercontent.com/zhangyunupupup/cn-codex/main/install.sh | bash

set -euo pipefail

# === 常量 ===
CN_CODEX_HOME="${CN_CODEX_HOME:-$HOME/.cn-codex}"
CN_CODEX_VERSION="0.1.0"
REPO_BASE="https://raw.githubusercontent.com/zhangyunupupup/cn-codex/main"

# === 颜色 ===
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

info()  { echo -e "${BLUE}ℹ${NC} $*"; }
ok()    { echo -e "${GREEN}✔${NC} $*"; }
warn()  { echo -e "${YELLOW}⚠${NC} $*"; }
error() { echo -e "${RED}✖${NC} $*" >&2; }

# === 检测平台 ===
detect_platform() {
    local os arch
    os="$(uname -s | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m)"

    case "$os" in
        linux)  os="linux" ;;
        darwin) os="macos" ;;
        mingw*|msys*|cygwin*) os="windows" ;;
        *) error "不支持的操作系统: $os"; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)  arch="aarch64" ;;
        *) error "不支持的架构: $arch"; exit 1 ;;
    esac

    echo "${os}-${arch}"
}

# === 安装 Codex CLI ===
install_codex_cli() {
    if command -v codex &>/dev/null; then
        ok "Codex CLI 已安装: $(codex --version 2>/dev/null || echo 'unknown')"
        return 0
    fi

    info "正在安装 Codex CLI..."

    if command -v npm &>/dev/null; then
        npm install -g @openai/codex
        ok "Codex CLI 安装成功"
    elif command -v brew &>/dev/null; then
        brew install codex
        ok "Codex CLI 安装成功"
    else
        error "需要 npm 或 brew 来安装 Codex CLI"
        error "请先安装 Node.js: https://nodejs.org/"
        return 1
    fi
}

# === 下载桥接代理 ===
install_bridge() {
    local platform
    platform=$(detect_platform)
    local bin_dir="${CN_CODEX_HOME}/bin"
    mkdir -p "$bin_dir"

    # 检查是否已有二进制
    if [[ -x "${bin_dir}/cn-codex-bridge" ]]; then
        ok "桥接代理已安装"
        return 0
    fi

    # 尝试从 GitHub Release 下载
    local ext=""
    local os_name
    os_name=$(echo "$platform" | cut -d'-' -f1)

    if [[ "$os_name" == "windows" ]]; then
        ext=".exe"
    fi

    local download_url="https://github.com/zhangyunupupup/cn-codex/releases/download/v${CN_CODEX_VERSION}/cn-codex-bridge-${platform}${ext}"

    info "正在下载桥接代理..."
    if command -v curl &>/dev/null; then
        curl -fsSL "$download_url" -o "${bin_dir}/cn-codex-bridge${ext}" 2>/dev/null || {
            warn "无法从 GitHub Release 下载，尝试本地编译..."
            install_bridge_from_source
            return
        }
    elif command -v wget &>/dev/null; then
        wget -q "$download_url" -O "${bin_dir}/cn-codex-bridge${ext}" 2>/dev/null || {
            warn "无法从 GitHub Release 下载，尝试本地编译..."
            install_bridge_from_source
            return
        }
    fi

    chmod +x "${bin_dir}/cn-codex-bridge${ext}"
    ok "桥接代理下载完成"
}

# === 从源码编译桥接代理 ===
install_bridge_from_source() {
    if ! command -v cargo &>/dev/null; then
        # 尝试安装 Rust
        info "正在安装 Rust 工具链..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    fi

    info "正在编译桥接代理（首次编译约需 2-5 分钟）..."

    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf '$tmp_dir'" EXIT

    # Clone 最小代码
    git clone --depth 1 https://github.com/zhangyunupupup/cn-codex.git "$tmp_dir/cn-codex" 2>/dev/null || {
        error "无法克隆仓库，请检查网络连接"
        return 1
    }

    cd "$tmp_dir/cn-codex/bridge"
    cargo build --release 2>&1 | tail -5

    cp target/release/cn-codex-bridge "${CN_CODEX_HOME}/bin/cn-codex-bridge"
    ok "桥接代理编译完成"
}

# === 安装 cn-codex 主脚本 ===
install_wrapper() {
    local bin_dir="${CN_CODEX_HOME}/bin"
    mkdir -p "$bin_dir"

    # 下载主脚本
    local scripts=("cn-codex" "cn-codex-setup")
    for script in "${scripts[@]}"; do
        info "安装 ${script}..."
        if command -v curl &>/dev/null; then
            curl -fsSL "${REPO_BASE}/${script}" -o "${bin_dir}/${script}"
        fi
        chmod +x "${bin_dir}/${script}"
    done

    # 添加到 PATH
    local shell_rc=""
    if [[ -f "$HOME/.bashrc" ]]; then
        shell_rc="$HOME/.bashrc"
    elif [[ -f "$HOME/.zshrc" ]]; then
        shell_rc="$HOME/.zshrc"
    fi

    if [[ -n "$shell_rc" ]]; then
        if ! grep -q "cn-codex/bin" "$shell_rc" 2>/dev/null; then
            echo "" >> "$shell_rc"
            echo "# cn-codex" >> "$shell_rc"
            echo "export PATH=\"${bin_dir}:\$PATH\"" >> "$shell_rc"
            ok "已添加 PATH 到 ${shell_rc}"
        fi
    fi

    # 创建符号链接到 /usr/local/bin（如果可以）
    if [[ -w /usr/local/bin ]]; then
        ln -sf "${bin_dir}/cn-codex" /usr/local/bin/cn-codex 2>/dev/null || true
        ln -sf "${bin_dir}/cn-codex-setup" /usr/local/bin/cn-codex-setup 2>/dev/null || true
    fi
}

# === 主流程 ===
main() {
    echo ""
    echo -e "${BOLD}${CYAN}🇨🇳 cn-codex v${CN_CODEX_VERSION} 安装程序${NC}"
    echo -e "${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # 1. 安装 Codex CLI
    install_codex_cli

    # 2. 安装桥接代理
    install_bridge

    # 3. 安装 wrapper 脚本
    install_wrapper

    # 完成
    echo ""
    echo -e "${BOLD}${GREEN}🇨🇳 安装完成！${NC}"
    echo ""
    echo "下一步："
    echo -e "  ${CYAN}1.${NC} 配置国产大模型："
    echo -e "     ${BOLD}cn-codex-setup${NC}"
    echo ""
    echo -e "  ${CYAN}2.${NC} 或直接使用："
    echo -e "     ${BOLD}cn-codex \"你的任务\"${NC}"
    echo ""
    echo -e "  ${DIM}提示: 重新加载 shell 以更新 PATH${NC}"
    echo -e "  ${DIM}source ~/.bashrc  # 或 source ~/.zshrc${NC}"
    echo ""
}

main "$@"
