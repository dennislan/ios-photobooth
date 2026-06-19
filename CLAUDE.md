# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**大头贴 (Photobooth)** — A macOS desktop photobooth app built with Tauri 2.0 + Vue 3 + Rust. It connects to an iPhone via USB for remote camera control, streams a live MJPEG preview from the phone's camera, captures close-up portraits (大头照), composites photos onto branded templates using FFmpeg, and prints the result. Designed for offline retail-store use with an OTA self-update mechanism.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2.0 (Rust backend + WebView) |
| Frontend | Vue 3 + Vite + Pinia + TypeScript + Ant Design Vue 4 + Tailwind CSS 4 |
| Backend | Rust (Tauri commands, async/tokio) |
| iPhone camera | libimobiledevice (`idevice_id`, `ideviceinfo`) for device discovery + Swift helper (AVFoundation) for MJPEG preview & photo capture |
| Media processing | FFmpeg (HEIC handling, HEVC transcoding, template compositing, ffprobe) |
| Printing | macOS `lpr` / Windows PowerShell / Linux `lp` |
| OTA updates | Self-hosted HTTP endpoint, semver comparison, in-place `.app` bundle replacement + restart helper |
| Deep link | `photobooth://` URL scheme |

## Architecture

```
photobooth/
├── src/                          # Vue 3 frontend
│   ├── main.ts                   # Entry — creates Vue app, mounts Pinia + Antd
│   ├── App.vue                   # Root — 4-view router (idle → capture → selection → preview) + UpdateBanner
│   ├── stores/                   # Pinia stores
│   │   ├── capture.ts            # Capture mode, photos list, device state
│   │   ├── camera.ts             # iPhone camera connection state (connected, running, deviceId)
│   │   ├── template.ts           # Template configs & composite status
│   │   └── update.ts             # OTA update state machine
│   ├── components/
│   │   ├── IdleView.vue          # Standby screen
│   │   ├── SettingsPanel.vue     # App settings modal
│   │   └── UpdateBanner.vue      # Update notification banner
│   ├── views/
│   │   ├── CaptureView.vue       # Live MJPEG preview + photo capture
│   │   ├── SelectionView.vue     # Photo selection
│   │   └── PreviewView.vue       # Template composite preview + print
│   └── styles/
│       └── tailwind.css          # Tailwind CSS entry + global styles
│
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── lib.rs                # Tauri entry — registers all commands & plugins, APP_VERSION
│   │   ├── ios_camera.rs         # iPhone discovery (idevice_id) + device info (ideviceinfo) + caching
│   │   ├── camera_stream.rs      # Swift helper process lifecycle (start/stop/is_running)
│   │   ├── ffmpeg.rs             # Live Photo detection, ffprobe photo info, HEVC→H.264 transcode
│   │   ├── template.rs           # JSON-driven template compositing (FFmpeg filter graph)
│   │   ├── printer.rs            # Cross-platform printing (lpr / PowerShell / cups)
│   │   ├── deep_link.rs          # photobooth:// URL parsing
│   │   ├── updater.rs            # Update manifest fetch (semver), download, in-place bundle replace
│   │   ├── restart.rs            # nohup survivor script — relaunches app after update
│   │   ├── utils.rs              # Shared helpers: path resolution, base64, executable lookup
│   │   └── main.rs               # Binary entry — calls lib::main()
│   ├── ios_camera_stream/        # Swift helper tool (Swift Package Manager)
│   │   ├── Package.swift
│   │   └── Sources/main.swift    # AVCaptureSession + MJPEG TCP server (port 27183) + stdin-driven capture
│   ├── resources/
│   │   └── ios_camera_stream     # Prebuilt Swift helper binary (bundled resource)
│   ├── Cargo.toml                # Dependencies: tauri 2, tokio, reqwest, serde, semver, ffmpeg via CLI; lib name: photobooth_lib
│   ├── tauri.conf.json           # App config: window 1280×900, output dir gen/, identifier com.photobooth.app
│   ├── capabilities/default.json # Permission grants for Tauri plugins
│   ├── build.rs                  # tauri_build::build()
│   └── icons/                    # App icons for all platforms
│
├── build.sh                      # One-click build script (checks prereqs, builds frontend + Tauri)
├── package.json                  # npm scripts: dev, build, tauri:dev, tauri:build
├── vite.config.ts                # Vue plugin, PostCSS, server on 0.0.0.0:1420, output to src-tauri/gen
├── postcss.config.js             # Tailwind CSS 4 PostCSS plugin
├── tsconfig.json                 # ES2020, strict, path alias @/* → src/*
└── index.html
```

### Key design patterns

- **4-view flow**: `App.vue` switches between `idle` → `capture` → `selection` → `preview` views via a tabbed header.
- **Swift helper process**: `camera_stream.rs` spawns the `ios_camera_stream` Swift executable as a child process. The helper runs an AVCaptureSession and serves an MJPEG stream over a local TCP socket (port 27183). The frontend consumes this stream via an `<img src="http://127.0.0.1:27183">` tag (multipart/x-mixed-replace). Photo capture is triggered by sending `"capture"` to the helper's stdin; saved HEIC files land in `/tmp/photobooth/`.
- **Async Tauri commands**: All backend work runs via `tokio::task::block_in_place()` to avoid blocking the WebView thread.
- **Executable lookup**: `utils::find_executable()` checks the app data dir first (`~/Library/Application Support/photobooth/`), then PATH — enables shipping bundled binaries (idevice_id, ffmpeg, ios_camera_stream).
- **Template compositing**: One JSON config drives both frontend Canvas preview (via `calculate_cover_crop`) and backend FFmpeg filter graphs (object-fit: cover equivalent).
- **OTA updates**: `updater.rs` fetches a JSON manifest, compares semver, downloads a zip, then `apply_update` renames the old `.app` aside, unzips the new bundle. `restart.rs` spawns a `nohup` survivor bash script that waits for the old process to exit, cleans up `.app.old`, and reopens the new app.

## Common Commands

### Development

```bash
npm install                          # Install frontend dependencies
npm run tauri:dev                    # Start Tauri dev server (Vue + WebView + Rust)
npm run dev                          # Start Vue dev server only (Vite on :1420)
npm run build                        # Build frontend only (type-check + Vite)
```

### Building the macOS app

```bash
# Prerequisites: Rust toolchain, Node.js, Tauri CLI, Xcode/Swift toolchain
./build.sh                           # One-click: checks prereqs, builds frontend + Tauri bundle
# or manually:
npm run tauri:build                  # Build production macOS .app bundle

# Output: src-tauri/target/release/bundle/macos/
```

The `build.sh` pipeline:
1. Checks for Node.js, Rust/Cargo, Tauri CLI, and code-signing certificates
2. Runs `npm install` (if `node_modules` missing)
3. Runs `npm run build` → compiles Vue/TS to `src-tauri/gen/`
4. Runs `tauri build` → compiles Rust backend + bundles into `.app`/`.dmg`

### Building the Swift camera helper

```bash
cd src-tauri/ios_camera_stream
swift build -c release               # Build the ios_camera_stream executable
# Copy the built binary to src-tauri/resources/ios_camera_stream for bundling
```

### TypeScript type-checking

```bash
npx vue-tsc --noEmit                 # Standalone type check
```

## Dependencies

### Frontend (package.json)
- **Runtime**: Vue 3.5+, Pinia 2.3, Ant Design Vue 4.2 (`@ant-design/icons-vue`), Tailwind CSS 4.3 (`@tailwindcss/postcss`), `@tauri-apps/*` plugins (api, http, shell, dialog, fs, opener)
- **Dev**: Vite 6.2, @vitejs/plugin-vue 5.2, TypeScript 5.8, vue-tsc 2.2, @tauri-apps/cli 2.5

### Backend (Cargo.toml)
- **Tauri**: tauri 2, tauri-plugin-opener/http/dialog/fs 2
- **Async**: tokio (full), reqwest 0.12
- **Serialization**: serde (derive), serde_json
- **Utilities**: uuid v4, chrono, base64, dirs 6, log, once_cell, thiserror 2, semver 1
- **Windows only**: windows 0.60 (Win32 API — legacy, not used in iOS flow)

## Build Prerequisites

The app requires external binaries available on PATH or in the app data dir (`~/Library/Application Support/photobooth/`):
- **libimobiledevice** — `idevice_id` (device discovery) and `ideviceinfo` (device info). macOS: `brew install libimobiledevice`
- **ffmpeg** — Media processing (transcoding, compositing, probing, thumbnail extraction). macOS: `brew install ffmpeg`
- **ios_camera_stream** — The Swift camera helper (built from `src-tauri/ios_camera_stream/`, bundled in `src-tauri/resources/`)

These are not bundled as npm/Cargo deps — they must be installed on the target machine or shipped in the app data directory / resources.

## Important Notes

- The frontend outputs to `src-tauri/gen/` (configured in both `vite.config.ts` and `tauri.conf.json`)
- Tauri dev server listens on `0.0.0.0:1420`
- CSP allows `unsafe-inline` for styles and scripts, and `http://127.0.0.1:27183` for MJPEG preview images
- App data dir: `~/Library/Application Support/photobooth/` (used for bundled binaries, update staging, restart helper script)
- Assets dir: `~/Desktop/photobooth-assets/` (macOS) — where composite outputs are saved
- Captured photos are written to `/tmp/photobooth/` as HEIC files
- The MJPEG preview stream is served on `127.0.0.1:27183` by the Swift helper
- OTA updates are sourced from the latest GitHub release. Repo configurable via env var `PHOTOBOOTH_GITHUB_REPO` (format: `owner/repo`)
- App identifier: `com.photobooth.app`, window title: `大头贴`
