#!/usr/bin/env bash
# cn-codex 一键安装脚本
#
# 用法:
#   curl -fsSL https://raw.githubusercontent.com/zhangyunupupup/cn-codex/main/install.sh | bash
#   # 或下载后: bash install.sh
#
# 注意: ❌ 不要双击运行！必须在终端中执行！
#       Windows: 打开 Git Bash → 输入 bash install.sh
#       Mac/Linux: 打开终端 → 输入 bash install.sh

# === 双击检测（Windows Git Bash 双击会闪退，提前拦截）===
detect_double_click() {
    # 仅在 Windows Git Bash (MINGW/MSYS/CYGWIN) 下检测
    case "$(uname -s 2>/dev/null)" in
        MINGW*|MSYS*|CYGWIN*)
            # 双击运行时 stdin 不是终端（TTY）
            if [[ ! -t 0 ]]; then
                echo ""
                echo "===================================================="
                echo "  ⚠️  检测到双击运行！"
                echo ""
                echo "  请勿双击 .sh 文件运行！"
                echo ""
                echo "  正确方式：打开 Git Bash 终端，输入："
                echo ""
                echo "    cd 文件所在目录"
                echo "    bash install.sh"
                echo ""
                echo "  此窗口将在 10 秒后自动关闭..."
                echo "===================================================="
                sleep 10
                exit 1
            fi
            ;;
    esac
}
detect_double_click

# 不使用 set -e，改为手动错误处理，避免无声闪退
set -uo pipefail

# === 常量 ===
CN_CODEX_HOME="${CN_CODEX_HOME:-$HOME/.cn-codex}"
CN_CODEX_VERSION="0.1.0"
REPO_BASE="https://raw.githubusercontent.com/zhangyunupupup/cn-codex/main"
BRIDGE_BIN="${CN_CODEX_HOME}/bin/cn-codex-bridge"

# === 颜色（兼容 Windows Git Bash） ===
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    CYAN='\033[0;36m'
    BOLD='\033[1m'
    DIM='\033[2m'
    NC='\033[0m'
else
    RED= GREEN= YELLOW= BLUE= CYAN= BOLD= DIM= NC=
fi

info()  { printf "${BLUE}ℹ${NC} %s\n" "$*"; }
ok()    { printf "${GREEN}✔${NC} %s\n" "$*"; }
warn()  { printf "${YELLOW}⚠${NC} %s\n" "$*"; }
error() { printf "${RED}✖${NC} %s\n" "$*" >&2; }

# === 检测是否在 Windows 原生环境下运行 ===
check_windows() {
    case "$(uname -s 2>/dev/null || echo 'unknown')" in
        MINGW*|MSYS*|CYGWIN*) return 0 ;;  # Git Bash / MSYS2 — 可用
        *) return 1 ;;
    esac
}

# === 安装 Codex CLI ===
install_codex_cli() {
    if command -v codex &>/dev/null; then
        local ver
        ver=$(codex --version 2>/dev/null || echo '已安装')
        ok "Codex CLI ${ver}"
        return 0
    fi

    info "正在安装 Codex CLI..."

    if command -v npm &>/dev/null; then
        npm install -g @openai/codex && {
            ok "Codex CLI 安装成功"
            return 0
        }
        warn "npm 安装失败，试试 Homebrew..."
    fi

    if command -v brew &>/dev/null; then
        brew install codex && {
            ok "Codex CLI 安装成功"
            return 0
        }
    fi

    cat << HELP

${YELLOW}╔══════════════════════════════════════════════╗
║  需要手动安装 Codex CLI                       ║
║                                              ║
║  方式一: npm install -g @openai/codex         ║
║  方式二: brew install codex                   ║
║  方式三: 从 https://github.com/openai/codex   ║
║          下载对应平台的二进制                  ║
╚══════════════════════════════════════════════╝${NC}
HELP
    return 1
}

# === 安装桥接代理 ===
install_bridge() {
    local bin_dir="${CN_CODEX_HOME}/bin"
    mkdir -p "$bin_dir"

    # 已有二进制则跳过
    if [[ -x "$BRIDGE_BIN" ]]; then
        ok "桥接代理已安装"
        return 0
    fi

    # 方案一：从 GitHub Release 下载（如果已发布）
    local platform=""
    local ext=""
    platform=$(detect_platform 2>/dev/null || true)
    if check_windows; then
        ext=".exe"
    fi

    if [[ -n "$platform" ]]; then
        local download_url="https://github.com/zhangyunupupup/cn-codex/releases/download/v${CN_CODEX_VERSION}/cn-codex-bridge-${platform}${ext}"
        info "尝试下载桥接代理..."
        if command -v curl &>/dev/null; then
            if curl -fsSL "$download_url" -o "$BRIDGE_BIN$ext" 2>/dev/null; then
                chmod +x "$BRIDGE_BIN$ext" 2>/dev/null || true
                ok "桥接代理下载完成"
                return 0
            fi
        elif command -v wget &>/dev/null; then
            if wget -q "$download_url" -O "$BRIDGE_BIN$ext" 2>/dev/null; then
                chmod +x "$BRIDGE_BIN$ext" 2>/dev/null || true
                ok "桥接代理下载完成"
                return 0
            fi
        fi
        warn "暂未从 GitHub Release 下载到（首次发布后可用）"
    fi

    # 方案二：从源码编译
    warn "尝试从源码编译桥接代理..."
    install_bridge_from_source
}

# === 检测平台（用于 Release 下载） ===
detect_platform() {
    local os arch
    os="$(uname -s 2>/dev/null | tr '[:upper:]' '[:lower:]')"
    arch="$(uname -m 2>/dev/null)"

    case "$os" in
        linux)  os="linux" ;;
        darwin) os="macos" ;;
        mingw*|msys*|cygwin*) os="windows" ;;
        *) echo ""; return 1 ;;
    esac

    case "$arch" in
        x86_64|amd64)   arch="x86_64" ;;
        aarch64|arm64)  arch="aarch64" ;;
        *) echo ""; return 1 ;;
    esac

    echo "${os}-${arch}"
}

# === 从源码编译桥接代理 ===
install_bridge_from_source() {
    # 检查 Rust
    if ! command -v cargo &>/dev/null; then
        warn "需要 Rust 工具链来编译桥接代理"
        printf "是否自动安装 Rust？[Y/n]: "
        read -r answer
        if [[ "${answer:-Y}" =~ ^[Yy] ]]; then
            info "正在安装 Rust..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y 2>/dev/null || {
                error "Rust 安装失败，请手动安装: https://rustup.rs"
                return 1
            }
            # shellcheck source=$HOME/.cargo/env
            source "$HOME/.cargo/env" 2>/dev/null || true
        else
            error "跳过编译。可手动下载二进制或安装 Rust 后重试。"
            return 1
        fi
    fi

    # 检查 git
    if ! command -v git &>/dev/null; then
        error "需要 git 来克隆仓库，请先安装 git"
        return 1
    fi

    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf '$tmp_dir'" EXIT

    info "正在克隆 cn-codex 仓库..."
    if ! git clone --depth 1 "https://github.com/zhangyunupupup/cn-codex.git" "$tmp_dir/cn-codex" 2>/dev/null; then
        error "克隆仓库失败，请检查网络连接"
        return 1
    fi

    info "正在编译桥接代理（首次编译约需 2-5 分钟）..."
    (
        cd "$tmp_dir/cn-codex/bridge"
        cargo build --release 2>&1
    ) || {
        error "编译失败，请检查错误信息"
        return 1
    }

    local built_bin="$tmp_dir/cn-codex/bridge/target/release/cn-codex-bridge"
    if [[ -f "$built_bin" ]]; then
        cp "$built_bin" "$BRIDGE_BIN"
        chmod +x "$BRIDGE_BIN"
        ok "桥接代理编译完成"
    else
        error "编译产物未找到"
        return 1
    fi
}

# === 安装 wrapper 脚本 ===
install_wrapper() {
    local bin_dir="${CN_CODEX_HOME}/bin"
    mkdir -p "$bin_dir"

    local scripts=("cn-codex" "cn-codex-setup")
    local all_ok=true

    for script in "${scripts[@]}"; do
        local target="${bin_dir}/${script}"
        if [[ -f "$target" ]]; then
            ok "${script} 已存在"
            continue
        fi
        info "下载 ${script}..."
        if command -v curl &>/dev/null; then
            if ! curl -fsSL "${REPO_BASE}/${script}" -o "$target" 2>/dev/null; then
                warn "下载 ${script} 失败（可手动从 GitHub 复制）"
                all_ok=false
                continue
            fi
        elif command -v wget &>/dev/null; then
            if ! wget -q "${REPO_BASE}/${script}" -O "$target" 2>/dev/null; then
                warn "下载 ${script} 失败（可手动从 GitHub 复制）"
                all_ok=false
                continue
            fi
        else
            warn "需要 curl 或 wget 来下载脚本"
            all_ok=false
            continue
        fi
        chmod +x "$target" 2>/dev/null || true
        ok "${script} 下载完成"
    done

    # 添加到 PATH
    local shell_rc=""
    if [[ -f "$HOME/.bashrc" ]]; then
        shell_rc="$HOME/.bashrc"
    elif [[ -f "$HOME/.zshrc" ]]; then
        shell_rc="$HOME/.zshrc"
    elif check_windows && [[ -f "$HOME/.bash_profile" ]]; then
        shell_rc="$HOME/.bash_profile"
    fi

    if [[ -n "$shell_rc" ]]; then
        if ! grep -q "cn-codex/bin" "$shell_rc" 2>/dev/null; then
            {
                echo ""
                echo "# cn-codex"
                echo "export PATH=\"${bin_dir}:\$PATH\""
            } >> "$shell_rc"
            ok "已添加 PATH 到 ${shell_rc}"
        fi
    else
        warn "未找到 shell 配置文件，请手动将以下内容添加到你的 shell 配置："
        echo "  export PATH=\"${bin_dir}:\$PATH\""
    fi

    if $all_ok; then
        return 0
    else
        return 1
    fi
}

# === 主流程 ===
main() {
    echo ""
    printf "${BOLD}${CYAN}🇨🇳 cn-codex v${CN_CODEX_VERSION} 安装程序${NC}\n"
    printf "${DIM}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
    echo ""

    local has_error=false

    # 1. 安装 Codex CLI
    install_codex_cli || has_error=true

    echo ""

    # 2. 安装桥接代理
    install_bridge || has_error=true

    echo ""

    # 3. 安装 wrapper 脚本
    install_wrapper || has_error=true

    echo ""
    if $has_error; then
        printf "${BOLD}${YELLOW}⚠ 安装完成（有警告）${NC}\n"
        echo ""
        echo "部分组件未成功安装，你可以稍后手动处理："
        echo "  - 桥接代理: 从 GitHub Releases 下载或手动编译"
        echo "  - Wrapper:  从 https://github.com/zhangyunupupup/cn-codex 复制"
        echo ""
        echo "缺少桥接代理不影响 cn-codex-setup 配置使用"
    else
        printf "${BOLD}${GREEN}🇨🇳 安装完成！${NC}\n"
    fi

    echo ""
    echo "下一步："
    printf "  ${CYAN}1.${NC} 配置国产大模型：\n"
    printf "     ${BOLD}cn-codex-setup${NC}\n"
    echo ""
    printf "  ${CYAN}2.${NC} 或直接使用：\n"
    printf "     ${BOLD}cn-codex \"你的任务\"${NC}\n"
    echo ""
    printf "  ${DIM}提示: 重新加载 shell 以更新 PATH${NC}\n"
    printf "  ${DIM}       source ~/.bashrc  # 或 source ~/.zshrc${NC}\n"
    echo ""
}

main "$@"
