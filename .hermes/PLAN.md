# android大头贴拍照体验系统 — 实施计划

## 项目概述
基于 Tauri 2.0 + Rust + Vue 3 开发线下门店「大头贴」拍照合成打印一体化桌面应用。

## 技术栈
| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2.0 (Rust) |
| 前端 | Vue 3 + Vite |
| 后端 | Rust (Tauri commands) |
| 投屏 | scrcpy + ADB |
| 视频处理 | FFmpeg |
| 模板合成 | Canvas (前端) + FFmpeg (后端) |
| 打印 | macOS lpr / Windows PowerShell System.Drawing |
| 更新 | Tauri Updater (Ed25519) |
| 协议 | Deep Link + Single Instance |

## 项目结构
```
android-photobooth/
├── src/                          # Tauri 后端
│   ├── lib.rs                    # 入口，注册插件和 commands
│   ├── scrcpy.rs                 # scrcpy 进程管理 + 窗口嵌入
│   ├── adb.rs                    # ADB 命令执行 + 多级缓存
│   ├── ffmpeg.rs                 # HEVC 转码 + Live Photo 检测
│   ├── template.rs               # 模板合成引擎 (Canvas/FFmpeg JSON)
│   ├── printer.rs                # 跨平台打印适配
│   ├── deep_link.rs              # Deep Link + 单实例
│   └── utils.rs                  # 工具函数
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   ├── gen/                      # 自动生成
│   ├── icons/                    # 应用图标
│   ├── build.rs
│   └── update-key.pub            # Ed25519 公钥
├── src/                          # Vue 前端
│   ├── main.ts
│   ├── App.vue
│   ├── components/
│   │   ├── ScrcpyView.vue        # 投屏显示区
│   │   ├── CaptureModeSelector.vue # 模式选择（四宫格/报纸机）
│   │   ├── PhotoGallery.vue      # 照片画廊（选中）
│   │   ├── TemplatePreview.vue   # 模板预览 (Canvas)
│   │   ├── PrintPanel.vue        # 打印面板
│   │   └── SettingsPanel.vue     # 设置面板
│   ├── views/
│   │   ├── IdleView.vue          # 待机界面
│   │   ├── CaptureView.vue       # 拍照界面
│   │   ├── SelectionView.vue     # 选片界面
│   │   └── PreviewView.vue       # 预览/打印界面
│   ├── stores/
│   │   ├── scrcpy.ts             # 投屏状态
│   │   ├── capture.ts            # 拍照状态
│   │   └── template.ts           # 模板状态
│   ├── services/
│   │   ├── tauri.ts              # Tauri IPC 封装
│   │   └── http.ts               # HTTP 请求 (无 CORS)
│   └── styles/
│       └── main.css
├── package.json
├── vite.config.ts
└── tsconfig.json
```

## 实施步骤

### Phase 1: 项目初始化 (约 15 min)
1. 创建 Tauri 项目 (`tauri init`)
2. 配置 Vue 3 + Vite 前端
3. 配置 Cargo.toml 依赖
4. 基础目录结构搭建

### Phase 2: Rust 后端核心模块 (约 60 min)
1. **scrcpy.rs** — scrcpy 进程管理、窗口嵌入、参数配置
2. **adb.rs** — ADB 命令执行、三级缓存策略
3. **ffmpeg.rs** — HEVC→H.264 转码、Live Photo 检测
4. **template.rs** — JSON 驱动的模板合成引擎
5. **printer.rs** — 跨平台打印适配
6. **deep_link.rs** — Deep Link + 单实例保护
7. **lib.rs** — 整合所有模块，注册 Tauri commands

### Phase 3: Vue 3 前端 (约 45 min)
1. **投屏界面** — 显示 scrcpy 窗口区域
2. **拍照模式选择** — 四宫格 / 报纸机
3. **照片画廊** — 缩略图浏览、选中
4. **模板预览** — Canvas 合成预览
5. **打印面板** — 纸张选择、打印执行
6. **状态管理** — Pinia stores

### Phase 4: 前后端集成 (约 30 min)
1. IPC 通信桥接
2. Deep Link 协议处理
3. 单实例保护
4. 异步架构 (spawn_blocking)

### Phase 5: 自动更新 (约 15 min)
1. Tauri Updater 插件配置
2. Ed25519 签名机制
3. 版本检查 UI

### Phase 6: 构建验证 (约 15 min)
1. `tauri dev` 启动开发
2. `tauri build` 构建发布
3. 功能测试

## 关键设计决策
- 一套代码双平台：20+ cfg 条件编译分支
- 异步架构：所有耗时操作 spawn_blocking
- 一份 JSON 配置驱动 Canvas + FFmpeg 双引擎
- object-fit: cover 双端一致性算法

## 风险与应对
- scrcpy/FFmpeg 需随应用打包分发 → 构建脚本自动拷贝
- Windows 窗口嵌入依赖 Win32 API → cfg 条件编译隔离
- Live Photo 转码稳定性 → 文件大小稳定性检测重试
