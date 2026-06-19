#!/usr/bin/env bash
# build.sh — 大头贴 (Photobooth) 一键构建脚本
# 构建 Swift 相机辅助工具 + 前端 + Tauri 后端，并验证产物
# 用法: ./build.sh [--release|--debug]
set -euo pipefail

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; BLUE='\033[0;34m'; NC='\033[0m'
info()    { printf "${BLUE}[INFO]${NC}    %s\n" "$*"; }
success() { printf "${GREEN}[OK]${NC}      %s\n" "$*"; }
warn()    { printf "${YELLOW}[WARN]${NC}    %s\n" "$*"; }
fail()    { printf "${RED}[FAIL]${NC}    %s\n" "$*"; exit 1; }

MODE="${1:---release}"
case "$MODE" in
  --release) SWIFT_OPT="release"; BUILD_LABEL="Release" ;;
  --debug)   SWIFT_OPT="debug";   BUILD_LABEL="Debug" ;;
  *)         fail "用法: $0 [--release|--debug]" ;;
esac

ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"

echo
info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
info "  大头贴 Photobooth — ${BUILD_LABEL} 构建"
info "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo

# Step 1: 前置环境检查
info "[1/6] 检查构建环境..."
command -v node >/dev/null 2>&1 || fail "Node.js 未安装 (https://nodejs.org)"
info "  Node.js    $(node --version)"
command -v npm >/dev/null 2>&1 || fail "npm 未安装"
info "  npm        $(npm --version)"
command -v cargo >/dev/null 2>&1 || fail "Rust/Cargo 未安装 (https://rustup.rs)"
info "  Rust       $(rustc --version)"
command -v swift >/dev/null 2>&1 || fail "Swift 未安装，请安装 Xcode CLT: xcode-select --install"
info "  Swift      $(swift --version 2>&1 | head -1)"
xcode-select -p >/dev/null 2>&1 || fail "Xcode CLT 未安装: xcode-select --install"
info "  Xcode CLT  已安装"
success "环境检查通过"
echo

# Step 2: 构建 Swift 相机辅助工具
info "[2/6] 构建 Swift 相机辅助工具 (ios_camera_stream)..."
SWIFT_DIR="${ROOT}/src-tauri/ios_camera_stream"
RESOURCES_DIR="${ROOT}/src-tauri/resources"
HELPER_BIN="${RESOURCES_DIR}/ios_camera_stream"
mkdir -p "${RESOURCES_DIR}"

# 清理可能残留旧路径的 Swift 构建缓存（项目重命名/迁移后 module cache 会失效）
if [ -d "${SWIFT_DIR}/.build" ]; then
  if grep -rl "vivo-photobooth" "${SWIFT_DIR}/.build" >/dev/null 2>&1; then
    info "  检测到旧路径缓存，清理 Swift 构建目录..."
    rm -rf "${SWIFT_DIR}/.build"
  fi
fi

cd "${SWIFT_DIR}"
if [ "${SWIFT_OPT}" = "release" ]; then
  swift build -c release 2>&1 || fail "Swift release 构建失败"
  SWIFT_OUTPUT=".build/release/ios_camera_stream"
else
  swift build 2>&1 || fail "Swift debug 构建失败"
  SWIFT_OUTPUT=".build/debug/ios_camera_stream"
fi
[ -f "${SWIFT_OUTPUT}" ] || fail "Swift 构建产物未找到: ${SWIFT_OUTPUT}"
cp "${SWIFT_OUTPUT}" "${HELPER_BIN}"
chmod +x "${HELPER_BIN}"
[ -x "${HELPER_BIN}" ] || fail "相机辅助工具不可执行: ${HELPER_BIN}"
success "相机辅助工具构建完成 ($(ls -lh "${HELPER_BIN}" | awk '{print $5}'))"
cd "${ROOT}"
echo

# Step 3: 安装前端依赖
info "[3/6] 检查前端依赖..."
if [ -d "node_modules" ]; then
  success "node_modules 已存在，跳过安装"
else
  info "  运行 npm install..."
  npm install 2>&1 || fail "npm install 失败"
  success "前端依赖安装完成"
fi
echo

# Step 4: 构建前端 (TypeScript 类型检查 + Vite 打包)
info "[4/6] 构建前端 (vue-tsc + vite)..."
npm run build 2>&1 || fail "前端构建失败 (类型检查或 Vite 打包)"
FRONTEND_DIST="${ROOT}/src-tauri/gen"
[ -d "${FRONTEND_DIST}" ] || fail "前端产物目录未生成: ${FRONTEND_DIST}"
[ -f "${FRONTEND_DIST}/index.html" ] || fail "前端产物 index.html 未找到"
success "前端构建完成 ($(find "${FRONTEND_DIST}" -type f | wc -l | tr -d ' ') 个文件)"
echo

# Step 5: 构建 Tauri 后端 (Rust 编译验证 + 打包)
info "[5/6] 构建 Tauri 后端 (cargo check + tauri build)..."
if command -v tauri >/dev/null 2>&1; then
  TAURI_CLI="$(command -v tauri)"
else
  TAURI_CLI="npx --yes @tauri-apps/cli"
fi
info "  Tauri CLI: ${TAURI_CLI}"
cd "${ROOT}/src-tauri"

# 检测并清理可能引用旧路径的 Rust 构建缓存
# （项目重命名/迁移后，tauri-build 生成的 permissions 缓存会残留旧路径）
if [ -d "target" ]; then
  STALE_PATH=$(grep -rl "vivo-photobooth" target/ 2>/dev/null | head -1)
  if [ -n "${STALE_PATH}" ]; then
    warn "  检测到构建缓存引用旧路径，清理 target/ 目录..."
    cargo clean 2>/dev/null || rm -rf target
  fi
fi

info "  运行 cargo check (Rust 编译验证)..."
cargo check 2>&1 || fail "Rust 编译检查失败 (cargo check)"
info "  运行 tauri build..."
if [[ "$TAURI_CLI" == npx* ]]; then
  eval $TAURI_CLI -- build 2>&1 || fail "Tauri 构建失败"
else
  $TAURI_CLI build 2>&1 || fail "Tauri 构建失败"
fi
cd "${ROOT}"
success "Tauri 后端构建完成"
echo

# Step 6: 验证构建产物
info "[6/6] 验证构建产物..."
BUNDLE_DIR="${ROOT}/src-tauri/target/release/bundle"
[ -d "${BUNDLE_DIR}" ] || fail "构建产物目录未找到: ${BUNDLE_DIR}"
APP_BUNDLE=$(find "${BUNDLE_DIR}" -name "*.app" -maxdepth 3 -type d 2>/dev/null | head -1)
[ -n "${APP_BUNDLE}" ] || fail "未找到 .app 构建产物"
APP_NAME=$(basename "${APP_BUNDLE}")
APP_SIZE=$(du -sh "${APP_BUNDLE}" | awk '{print $1}')

echo
success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
success "  构建成功！"
success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
success "  应用: ${APP_NAME}"
success "  路径: ${APP_BUNDLE}"
success "  大小: ${APP_SIZE}"
success "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 验证 .app 包内包含相机辅助工具
HELPER_IN_BUNDLE="${APP_BUNDLE}/Contents/Resources/resources/ios_camera_stream"
if [ -f "${HELPER_IN_BUNDLE}" ]; then
  success "  ✓ 相机辅助工具已包含在应用包内"
else
  warn "  ⚠ 相机辅助工具未在应用包内找到 (可能影响运行时相机功能)"
fi

echo
info "构建耗时完成 ✓"
