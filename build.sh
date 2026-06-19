#!/usr/bin/env bash
# build.sh — 一键构建 photobooth 应用
# 用法: ./build.sh [--release|--debug]
set -euo pipefail

# ── 颜色 ──────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
info()   { printf "${BLUE}[INFO]${NC}   %s\n" "$*"; }
success(){ printf "${GREEN}[OK]${NC}     %s\n" "$*"; }
warn()   { printf "${YELLOW}[WARN]${NC}   %s\n" "$*"; }
fail()   { printf "${RED}[FAIL]${NC}  %s\n" "$*"; exit 1; }

# ── 参数 ──────────────────────────────────────────────
MODE="${1:---release}"
case "$MODE" in
  --release) BUILD_MODE="Release" ;;
  --debug)   BUILD_MODE="Debug" ;;
  *)         fail "用法: $0 [--release|--debug]";;
esac

# ── 根目录 ────────────────────────────────────────────
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

# ── 前置检查 ──────────────────────────────────────────
info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
info "  photobooth $BUILD_MODE Build"
info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# 1) Node.js / npm
command -v node    >/dev/null 2>&1 || fail "Node.js 未安装"
command -v npm     >/dev/null 2>&1 || fail "npm 未安装"
NODE_VER=$(node --version)
NPM_VER=$(npm --version)
info "Node.js  $NODE_VER"
info "npm     $NPM_VER"

# 2) Rust / Cargo
command -v cargo   >/dev/null 2>&1 || fail "Rust/Cargo 未安装"
RUST_VER=$(rustc --version)
CARGO_VER=$(cargo --version)
info "Rust      $RUST_VER"
info "Cargo     $CARGO_VER"

# 3) Tauri CLI — resolve to absolute path to avoid npx swallowing --release
if command -v tauri >/dev/null 2>&1; then
  TAURI_CLI="$(command -v tauri)"
else
  # Use npx with -- to prevent npx from consuming --release
  TAURI_CLI="npx --yes @tauri-apps/cli"
fi
info "Tauri CLI   $TAURI_CLI"

# 4) 签名证书
DIST_CERT=$(security find-identity -v -p codesigning | grep "Apple Distribution" | head -1 | sed 's/.*"\(.*\)".*/\1/')
DEV_CERT=$(security find-identity -v -p codesigning | grep "Apple Development:" | head -1 | sed 's/.*"\(.*\)".*/\1/')
if [ -n "$DIST_CERT" ]; then
  info "Distribution 证书  $DIST_CERT"
else
  warn "未找到 Apple Distribution 证书 — 将使用 Development 签名"
fi

echo

# ── Step 2: 安装依赖 ─────────────────────────────────
info "[2/4] 检查前端依赖..."
if [ -d "node_modules" ]; then
  success "node_modules 已存在，跳过 npm install"
else
  npm install
  success "依赖安装完成"
fi

echo

# ── Step 3: 构建前端 ─────────────────────────────────
info "[3/4] 构建前端..."
npm run build
success "前端构建完成"

echo

# ── Step 4: Tauri 打包 ───────────────────────────────
info "[4/4] Tauri 打包 ($BUILD_MODE)..."
echo

if [ -n "$DIST_CERT" ]; then
  export TAURI_SIGNER_PRIVATE_KEY="*"
  export TAURI_SIGNER_PRIVATE_KEY_TYPE="certificate"
fi

# When TAURI_CLI uses npx, the -- separates npx's own args from tauri's args.
# When it's an absolute path, -- is passed through as-is.
# `tauri build` is always release; `tauri dev -- --release` is for debug builds.
if [[ "$TAURI_CLI" == npx* ]]; then
  eval $TAURI_CLI -- build
else
  $TAURI_CLI build
fi

# For debug builds, use `tauri dev -- --release` instead
if [ "$BUILD_MODE" = "Debug" ]; then
  info "Running in debug mode via tauri dev..."
  if [[ "$TAURI_CLI" == npx* ]]; then
    eval $TAURI_CLI -- dev -- --release
  else
    $TAURI_CLI dev -- --release
  fi
fi

echo

# ── 输出结果 ─────────────────────────────────────────
BUNDLE_DIR="$ROOT/src-tauri/target/$BUILD_MODE/bundle"

if [ -d "$BUNDLE_DIR" ]; then
  success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  success "  构建完成！产物位于:"
  success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  find "$BUNDLE_DIR" -maxdepth 2 -type f \( -name "*.dmg" -o -name "*.app" -o -name "*.pkg" -o -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" -o -name "*.msi" \) -exec ls -lh {} \; | awk '{print "  " $NF " (" $5 ")"}'
else
  warn "未找到构建产物目录: $BUNDLE_DIR"
fi

echo
info "耗时完成 ✓"
