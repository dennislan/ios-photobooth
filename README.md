# 📸 大头贴 Photobooth

**线下门店大头贴拍照体验系统** — 一款基于 Tauri 2.0 构建的 macOS 桌面应用程序，通过 USB 连接 iPhone 进行远程相机控制，实现实时预览、人像拍摄、品牌模板合成及照片打印。

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)
![Vue](https://img.shields.io/badge/Vue-3.5-4FC08D?logo=vuedotjs&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-1.70+-DEA584?logo=rust&logoColor=white)
![License](https://img.shields.io/badge/License-MIT-green)

## ✨ 功能特性

### 核心功能
- **📱 iPhone 远程控制** — 通过 USB 连接 iPhone，使用 `libimobiledevice` 自动发现设备并获取设备信息
- **🎥 实时 MJPEG 预览** — Swift 原生 AVFoundation 驱动的本地视频流（TCP 端口 27183），低延迟实时预览
- **📸 高清人像拍摄** — 支持拍摄 HEIC 格式照片，自动检测 Live Photo 并提取静态帧
- **🖼️ 品牌模板合成** — JSON 驱动的 FFmpeg 滤镜图合成引擎，支持自定义边框、背景与布局
- **🖨️ 跨平台打印** — 支持 macOS `lpr`、Windows PowerShell 及 Linux `lp` 打印输出
- **🔄 OTA 在线更新** — 基于 Semver 版本比较的自动更新机制，支持原地替换 `.app` 包并自动重启

### 技术亮点
- **三视图流程架构** — 拍摄 → 选片 → 预览打印的无缝切换体验
- **异步后端处理** — 全链路 `tokio` 异步执行，确保 WebView 线程永不阻塞
- **Deep Link 支持** — 注册 `photobooth://` URL Scheme，支持外部唤起
- **可扩展资源查找** — 优先加载应用数据目录内的二进制文件（`idevice_id`、`ffmpeg`、`ios_camera_stream`），支持离线部署

## 🛠️ 环境要求

### 运行环境
| 依赖 | 版本要求 | 用途 |
|------|----------|------|
| **macOS** | 12.0+ (Monterey 或更新版本) | 主操作系统 |
| **Node.js** | ≥ 18.0 | 前端构建与开发服务器 |
| **Rust / Cargo** | ≥ 1.70 | Tauri 后端编译 |
| **Xcode Command Line Tools** | 最新版 | Swift 工具链编译 |

### 外部二进制依赖（运行时必需）
| 工具 | 安装方式 (macOS) | 说明 |
|------|------------------|------|
| [libimobiledevice](https://libimobiledevice.org/) | `brew install libimobiledevice` | 提供 `idevice_id` / `ideviceinfo` 用于 iPhone 设备发现 |
| [FFmpeg](https://ffmpeg.org/) | `brew install ffmpeg` | 提供媒体转码、滤镜合成、缩略图提取能力 |
| `ios_camera_stream` | 从源码编译（见下方） | Swift 编写的 AVCaptureSession MJPEG 流服务端 |

> 💡 这些工具可通过 `PATH` 系统变量或应用数据目录 (`~/Library/Application Support/photobooth/`) 加载。

## 📦 项目技术栈

| 层级 | 技术 |
|------|------|
| **桌面框架** | Tauri 2.0 (Rust 后端 + WebView) |
| **前端** | Vue 3.5 · Vite 6 · Pinia · TypeScript · Ant Design Vue 4 · Tailwind CSS 4 |
| **后端** | Rust (Tauri Commands, async/tokio) |
| **iPhone 相机控制** | libimobiledevice (设备发现) + Swift/AVFoundation (MJPEG 预览 & 拍照) |
| **媒体处理** | FFmpeg (HEIC 处理、HEVC→H.264 转码、模板合成) |
| **打印** | macOS `lpr` / Windows PowerShell / Linux `lp` |
| **OTA 更新** | 自托管 HTTP 接口、Semver 比较、原地 `.app` 替换 + 重启守护脚本 |

## 🚀 快速开始

### 1. 克隆仓库

```bash
git clone https://github.com/your-org/ios-photobooth.git
cd ios-photobooth
```

### 2. 安装前端依赖

```bash
npm install
```

### 3. 开发模式运行

```bash
# 启动完整 Tauri 开发环境（Vue HMR + WebView + Rust 热重载）
npm run tauri:dev
```

应用窗口将自动打开，默认尺寸 **1280×900**。

### 4. 构建生产版本

#### 方式一：一键构建脚本（推荐）

```bash
# Release 构建（生产发布）
./build.sh --release

# Debug 构建（调试用）
./build.sh --debug
```

构建脚本会自动完成：
1. ✅ 检测 Node.js / Rust / Tauri CLI / 代码签名证书
2. ✅ 安装 npm 依赖（如需要）
3. ✅ TypeScript 类型检查 + Vite 前端构建
4. ✅ Tauri Rust 后端编译 + 打包为 `.app` / `.dmg`

#### 方式二：手动逐步构建

```bash
# 步骤 1: 构建前端
npm run build

# 步骤 2: Tauri 打包
npm run tauri:build
```

#### 构建产物位置

```
src-tauri/target/release/bundle/macos/
├── photobooth.app          # 应用程序包
└── photobooth_1.0.0_aarch64.dmg  # 安装映像
```

### 5. 编译 Swift 相机助手（可选）

如需修改或重新编译 iOS 相机流服务端：

```bash
cd src-tauri/ios_camera_stream
swift build -c release

# 将编译产物复制到 Tauri 资源目录
cp ./.build/release/ios_camera_stream ../resources/ios_camera_stream
```

## 📖 使用指南

### 基本工作流程

1. **连接 iPhone** — 使用数据线将 iPhone 连接到 Mac，并在手机上点击「信任」
2. **启动应用** — 运行 Photobooth，系统将自动发现已连接的 iPhone 设备
3. **实时预览** — 进入拍摄视图，通过 Swift 驱动的 MJPEG 流查看实时画面
4. **拍照** — 点击捕获按钮，照片以 HEIC 格式保存至 `/tmp/photobooth/`
5. **选片** — 在选择视图中挑选满意的照片
6. **预览 & 合成** — 选择品牌模板，FFmpeg 自动进行图像合成处理
7. **打印** — 点击打印按钮，输出至已配置的打印机

### 可用的 NPM Scripts

| 命令 | 说明 |
|------|------|
| `npm run dev` | 仅启动 Vite 前端开发服务器 (http://localhost:1420) |
| `npm run build` | TypeScript 类型检查 + 生产构建前端 |
| `npm run preview` | 预览生产构建的前端 |
| `npm run tauri:dev` | 完整 Tauri 开发模式（含热重载） |
| `npm run tauri:build` | 构建生产版本 Tauri 应用 |

### 配置说明

#### 应用配置 (`src-tauri/tauri.conf.json`)

| 配置项 | 默认值 | 说明 |
|--------|--------|------|
| `productName` | `"photobooth"` | 产品名称 |
| `identifier` | `"com.photobooth.app"` | 应用 Bundle ID |
| `window.title` | `"大头贴"` | 窗口标题 |
| `window.size` | `1280×900` | 默认窗口尺寸 |

#### OTA 更新源

OTA 更新默认从 GitHub Releases 获取，可通过环境变量自定义仓库地址：

```bash
export PHOTOBOOTH_GITHUB_REPO="your-org/your-repo"
```

格式为 `owner/repo`。

#### 重要路径

| 路径 | 说明 |
|------|------|
| `~/Library/Application Support/photobooth/` | 应用数据目录（存放捆绑二进制、更新暂存等） |
| `~/Desktop/photobooth-assets/` | 合成后的图片输出目录 |
| `/tmp/photobooth/` | 拍摄原始照片临时存放（HEIC 格式） |
| `127.0.0.1:27183` | MJPEG 预览流监听地址 |

### Deep Link

应用注册了 `photoboth://` URL Scheme，支持通过链接直接唤起应用：

```bash
open "photobooth://action?param=value"
```

## 🏗️ 项目结构

```
ios-photobooth/
├── src/                              # Vue 3 前端源码
│   ├── main.ts                       # 入口文件 — 创建 Vue 实例，挂载 Pinia + Antd
│   ├── App.vue                       # 根组件 — 四视图路由 + 更新横幅
│   ├── stores/                       # Pinia 状态管理
│   │   ├── camera.ts                 # iPhone 相机连接状态
│   │   ├── capture.ts                # 拍摄模式、照片列表、设备状态
│   │   ├── template.ts               # 模板配置与合成状态
│   │   └── update.ts                 # OTA 更新状态机
│   ├── components/                   # 公共组件
│   │   ├── SettingsPanel.vue         # 设置面板
│   │   └── UpdateBanner.vue          # 更新通知栏
│   ├── views/                        # 页面视图
│   │   ├── CaptureView.vue           # 实时预览 + 拍照
│   │   ├── SelectionView.vue         # 照片选择
│   │   └── PreviewView.vue           # 模板合成预览 + 打印
│   └── styles/
│       └── tailwind.css              # Tailwind CSS 入口 + 全局样式
│
├── src-tauri/                        # Rust 后端 + Tauri 配置
│   ├── src/
│   │   ├── lib.rs                    # Tauri 入口 — 注册所有命令和插件
│   │   ├── main.rs                   # 二进制入口
│   │   ├── ios_camera.rs             # iPhone 设备发现与信息获取
│   │   ├── camera_stream.rs          # Swift 辅助进程生命周期管理
│   │   ├── ffmpeg.rs                 # Live Photo 检测、转码、ffprobe
│   │   ├── template.rs               # JSON 驱动模板合成（FFmpeg 滤镜图）
│   │   ├── printer.rs                # 跨平台打印适配
│   │   ├── deep_link.rs              # URL Scheme 解析
│   │   ├── updater.rs                # OTA 更新逻辑（下载、Semver、替换）
│   │   ├── restart.rs                # nohup 守护重启脚本
│   │   └── utils.rs                  # 共享工具函数
│   ├── ios_camera_stream/            # Swift 相机助手（Swift Package Manager）
│   │   ├── Package.swift
│   │   └── Sources/main.swift        # AVCaptureSession + MJPEG TCP 服务端
│   ├── resources/
│   │   └── ios_camera_stream         # 预编译 Swift 辅助二进制文件
│   ├── Cargo.toml                    # Rust 依赖清单
│   ├── tauri.conf.json               # Tauri 应用配置
│   └── icons/                        # 多平台应用图标
│
├── build.sh                          # 一键构建脚本
├── package.json                      # NPM 依赖与脚本
├── vite.config.ts                    # Vite 构建配置
├── postcss.config.js                 # PostCSS (Tailwind CSS 4) 配置
├── tsconfig.json                     # TypeScript 配置（路径别名 @/* → src/*）
└── index.html                        # HTML 入口
```

## 🔧 开发注意事项

### 前端规范
- **禁止**使用 `as any`、`@ts-ignore`、`@ts-expect-error`
- 使用 Pinia stores 进行全局状态管理
- 组件采用 Vue 3 Composition API (`<script setup>`)

### 后端规范
- **禁止**在生产代码中使用 `unwrap()` 或 `panic!()`，统一使用 `?` 操作符传播错误
- 所有 Tauri Command 必须使用 `tokio::task::block_in_place()` 避免阻塞 WebView
- 二进制文件查找统一使用 `utils::find_executable()`

### 安全策略
- CSP 已配置为允许 `http://127.0.0.1:27183` 用于 MJPEG 预览图像加载
- 前端构建输出目录固定为 `src-tauri/gen/`

## 📄 License

本项目采用 **MIT License** 开源协议。

> ⚠️ 注意：项目当前未包含 LICENSE 文件，请在部署前添加合适的开源许可证文件。

---

<p align="center">
  <strong>Photobooth</strong> — 让每一次快门都成为美好的记忆 📸
</p>
