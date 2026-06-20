# CLAUDE.md

大头贴 (Photobooth) — macOS 桌面拍照打印应用。使用 Tauri 2.0 + Vue 3 + Rust + Swift 构建。

## 概述

通过 USB 连接 iPhone 作为外接相机，提供实时 MJPEG 预览、拍照、照片选择、Canvas 合成排版和打印功能。参照 macOS "Photo Booth" 核心体验设计。

## 技术栈

| 层 | 技术 |
|----|------|
| 桌面壳 | Tauri 2.0 (Rust + WebView) |
| 前端 | Vue 3 + Vite + Pinia + TypeScript + Ant Design Vue 4 + Tailwind CSS 4 |
| 后端 | Rust (Tauri commands, tokio) |
| 相机 | libimobiledevice + Swift (AVFoundation MJPEG 流) |
| 打印 | macOS lpr (CUPS) |

## 架构

```
photobooth/
├── src/                      # Vue 3 前端
│   ├── main.ts               # 入口
│   ├── App.vue               # 根组件 — 3 视图路由 (capture→select→print)
│   ├── stores/
│   │   ├── capture.ts        # 照片、布局、滤镜、选片、打印设置
│   │   └── camera.ts         # 相机连接状态
│   ├── components/
│   │   └── SettingsModal.vue # 设置弹窗
│   ├── views/
│   │   ├── CaptureView.vue   # 实时预览 + 拍照
│   │   ├── SelectView.vue    # 选片
│   │   └── PrintView.vue     # 合成预览 + 打印
│   └── styles/tailwind.css
├── src-tauri/                # Rust 后端
│   ├── src/
│   │   ├── lib.rs            # Tauri 入口 — 注册命令
│   │   ├── camera.rs         # iPhone 设备发现与验证
│   │   ├── stream.rs         # Swift helper 生命周期
│   │   ├── photo.rs          # 照片文件 I/O
│   │   ├── printer.rs        # macOS lpr 打印
│   │   ├── utils.rs          # 共享工具函数
│   │   └── main.rs           # 二进制入口
│   ├── ios_camera_stream/    # Swift 相机辅助工具
│   │   ├── Package.swift
│   │   └── Sources/main.swift
│   ├── resources/            # 预构建的 Swift helper 二进制
│   ├── Cargo.toml
│   └── tauri.conf.json
├── build.sh                  # 一键构建脚本 (6 步验证)
└── package.json
```

## Tauri 命令

| 命令 | 模块 | 功能 |
|------|------|------|
| `discover_devices` | camera.rs | 枚举已连接 iPhone |
| `get_device_info` | camera.rs | 获取设备名称/版本 |
| `start_camera` | stream.rs | 启动 MJPEG 预览流 |
| `stop_camera` | stream.rs | 停止预览流 |
| `is_camera_active` | stream.rs | 检查流状态 |
| `capture_photo` | stream.rs | 拍照，返回文件路径 |
| `read_photo` | photo.rs | 读取 JPEG → base64 |
| `save_print_image` | photo.rs | 保存 base64 → 临时文件 |
| `print_photo` | printer.rs | lpr 打印 |
| `diagnose_connection` | lib.rs | 连接诊断 |

## 关键设计

- **Swift helper 进程**：`stream.rs` 启动 `ios_camera_stream` 子进程，运行 AVCaptureSession 并在 `127.0.0.1:27183` 提供 MJPEG 流。前端通过 `<img src="http://127.0.0.1:27183">` 消费。拍照通过 stdin 发送 `"capture"`，JPEG 保存到 `/tmp/photobooth/`。
- **无 FFmpeg 依赖**：照片为 JPEG 格式，直接读取 + base64 编码即可。
- **Canvas 合成**：`PrintView.vue` 渲染高分辨率 Canvas (1200×1800)，object-fit: cover + CSS 滤镜，输出 → base64 → 临时文件 → lpr 打印。
- **异步命令**：所有后端工作使用 `tokio::task::block_in_place()`。

## 开发命令

```bash
npm install              # 安装前端依赖
npm run tauri:dev        # 启动 Tauri 开发服务器
npm run dev              # 仅 Vue 开发服务器 (:1420)
npm run build            # 构建前端 (类型检查 + Vite)
./build.sh               # 一键构建 (Swift + 前端 + Tauri + 验证)
```

## 构建脚本 (build.sh)

6 步完整构建流程，每步含验证：
1. 环境检查 (Node, npm, Rust, Swift, Xcode CLT)
2. 构建 Swift 相机辅助工具 → 复制到 resources/
3. 安装前端依赖
4. 构建前端 (vue-tsc --noEmit + Vite)
5. 构建 Tauri 后端 (cargo check + tauri build)
6. 验证 .app 产物和内置相机辅助工具

## 运行时依赖

需在 PATH 或应用数据目录 (`~/Library/Application Support/photobooth/`) 提供：
- **libimobiledevice** — `idevice_id` + `ideviceinfo` (macOS: `brew install libimobiledevice`)
- **ios_camera_stream** — Swift 相机辅助工具 (由 build.sh 构建并打包)

## 重要说明

- 前端产物输出到 `src-tauri/gen/`
- CSP 允许 `http://127.0.0.1:27183` (MJPEG 预览)
- 应用数据目录：`~/Library/Application Support/photobooth/`
- 临时照片目录：`/tmp/photobooth/`
- MJPEG 预览端口：`127.0.0.1:27183`
- 应用标识：`com.photobooth.app`，窗口标题：`大头贴`

## 反模式（禁止）

- `as any`、`@ts-ignore`、`@ts-expect-error`
- Rust 生产代码中使用 `unwrap()` 或 `panic!`
- 阻塞 WebView 线程的同步 I/O
- 硬编码路径（使用 `utils::find_executable` / Tauri `resolve_path`）
