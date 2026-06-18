# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**android大头贴** — A offline retail-store photobooth desktop app built with Tauri 2.0 + Vue 3 + Rust. It connects to Android devices via ADB for remote camera control, mirrors the phone screen via scrcpy, composites photos onto branded templates using FFmpeg, and prints the result.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2.0 (Rust backend + WebView) |
| Frontend | Vue 3 + Vite + Pinia + TypeScript |
| Backend | Rust (Tauri commands, async/tokio) |
| Phone mirroring | scrcpy + ADB |
| Media processing | FFmpeg (HEVC transcoding, template compositing, Live Photo handling) |
| Printing | macOS `lpr` / Windows PowerShell / Linux `lp` |
| Deep link | `android-photo://` URL scheme |

## Architecture

```
android-photobooth/
├── src/                          # Vue 3 frontend
│   ├── main.ts                   # Entry — creates Vue app, mounts Pinia
│   ├── App.vue                   # Root — 4-view router (idle → capture → selection → preview)
│   ├── stores/                   # Pinia stores
│   │   ├── capture.ts            # Capture mode, photos list, device state
│   │   ├── scrcpy.ts             # scrcpy connection state & config
│   │   └── template.ts           # Template configs & composite status
│   └── styles/
│       └── main.css              # Global dark-theme CSS variables
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs                # Tauri entry — registers all commands & plugins
│   │   ├── scrcpy.rs             # scrcpy process lifecycle + Windows window embedding
│   │   ├── adb.rs                # ADB: device discovery, photo list, thumbnail extraction, capture trigger
│   │   ├── ffmpeg.rs             # HEVC→H.264 transcode, Live Photo detection, ffprobe
│   │   ├── template.rs           # JSON-driven template compositing (FFmpeg filter graph)
│   │   ├── printer.rs            # Cross-platform printing (lpr / PowerShell / cups)
│   │   ├── deep_link.rs          # android-photo:// URL parsing + single-instance params
│   │   └── utils.rs              # Shared helpers: path resolution, base64, executable lookup
│   ├── Cargo.toml                # Dependencies: tauri 2, tokio, reqwest, serde, ffmpeg bindings via CLI
│   ├── tauri.conf.json           # App config: window size 1280×900, output dir ../gen
│   ├── capabilities/default.json # Permission grants for Tauri plugins
│   ├── build.rs                  # tauri_build::build()
│   └── icons/                    # (missing — needs to be created for build)
│
├── package.json                  # npm scripts: dev, build, tauri:dev, tauri:build
├── vite.config.ts                # Vue plugin, server on 0.0.0.0:1420, output to ../src-tauri/gen
├── tsconfig.json                 # ES2020, strict, path alias @/* → src/*
└── index.html
```

### Key design patterns

- **4-view flow**: App.vue switches between `idle` → `capture` → `selection` → `preview` views.
- **Async Tauri commands**: All backend work runs via `tokio::task::block_in_place()` to avoid blocking the WebView thread.
- **Conditional compilation**: Windows-specific scrcpy window embedding uses `#[cfg(target_os = "windows")]`; printing uses `#[cfg(target_os = "macos")]` / `windows` / `linux`.
- **Executable lookup**: `utils::find_executable()` checks app data dir first, then PATH — enables shipping bundled scrcpy/ffmpeg/adb.
- **Template compositing**: One JSON config drives both frontend Canvas preview (via `calculate_cover_crop`) and backend FFmpeg filter graphs.

## Missing Frontend Components

The following Vue components referenced in `App.vue` do **not** exist yet and need to be created:

- `src/components/IdleView.vue`
- `src/views/CaptureView.vue`
- `src/views/SelectionView.vue`
- `src/views/PreviewView.vue`
- `src/components/SettingsPanel.vue`

Also missing: `src/services/tauri.ts` and `src/services/http.ts` (referenced in PLAN.md).

## Common Commands

### Development

```bash
npm install                          # Install frontend dependencies
npm run tauri:dev                    # Start Tauri dev server (Vue + WebView)
npm run dev                          # Start Vue dev server only (Vite on :1420)
npm run build                        # Build frontend only (type-check + Vite)
```

### Building the macOS app

```bash
# Prerequisites: Rust toolchain, Node.js, Tauri CLI
# Then:
npm run tauri:build                  # Build production macOS .app bundle

# Output location: src-tauri/target/release/bundle/macos/android-photobooth.app
```

The build pipeline:
1. Runs `npm run build` → compiles Vue/TS to `src-tauri/gen/`
2. Runs `cargo build --release` → compiles Rust backend
3. Bundles everything into a macOS `.app` bundle with code signing

### TypeScript type-checking

```bash
npx vue-tsc --noEmit                 # Standalone type check
```

## Dependencies

### Frontend (package.json)
- **Runtime**: Vue 3.5+, Pinia 2.3, `@tauri-apps/*` plugins (api, http, shell, dialog, fs, opener)
- **Dev**: Vite 6.2, @vitejs/plugin-vue 5.2, TypeScript 5.8, vue-tsc 2.2

### Backend (Cargo.toml)
- **Tauri**: tauri 2, tauri-plugin-opener/http/dialog/fs 2
- **Async**: tokio (full), reqwest 0.12
- **Serialization**: serde (derive), serde_json
- **Utilities**: uuid v4, chrono, base64, dirs 6, log, once_cell, thiserror 2
- **Windows only**: windows 0.60 (Win32 API for window embedding)

## Build Prerequisites

The app requires external binaries to be available on PATH or in the app data dir:
- **adb** — Android Debug Bridge
- **scrcpy** — Screen mirroring
- **ffmpeg** — Media processing (transcoding, compositing, probing)

These are not bundled as npm/Cargo deps — they must be installed on the target machine or copied into the app data directory at runtime.

## Important Notes

- The frontend outputs to `src-tauri/gen/` (configured in both `vite.config.ts` and `tauri.conf.json`)
- Tauri dev server listens on `0.0.0.0:1420`
- CSP allows `unsafe-inline` for styles and scripts (template rendering requirement)
- Icons directory (`src-tauri/icons/`) is missing — `tauri build` will fail until icons are added
- The app uses `dirs::desktop_dir()` for assets on macOS (non-Windows path in `utils.rs`)
