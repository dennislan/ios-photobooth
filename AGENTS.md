# AGENTS.md

**Project:** 大头贴 Photobooth — macOS desktop photobooth app
**Stack:** Tauri 2.0 · Vue 3 · Rust · Swift
**Updated:** 2026-06-19

## STRUCTURE

```
photobooth/
├── src/              # Vue 3 frontend (views, components, stores, styles)
├── src-tauri/        # Rust backend + Tauri config + Swift camera helper
│   ├── src/          #   Rust modules (7 files, Tauri commands)
│   └── ios_camera_stream/  # Swift helper (AVCaptureSession → MJPEG)
├── build.sh          # One-click macOS builder (6-step verification)
└── index.html        # Vite entry
```

## WHERE TO LOOK

| Task | File | Notes |
|------|------|-------|
| App entry | `src/main.ts` | Creates Vue app, mounts Pinia + Antd |
| Root component | `src/App.vue` | 3-view router switching |
| Camera state | `src/stores/camera.ts` | Connection state, deviceId |
| Capture logic | `src/stores/capture.ts` | Photos, layout, filter, selection, print |
| Tauri entry | `src-tauri/src/lib.rs` | Registers all commands |
| Device discovery | `src-tauri/src/camera.rs` | idevice_id/ideviceinfo |
| Stream lifecycle | `src-tauri/src/stream.rs` | Swift helper spawn/monitor |
| Photo file I/O | `src-tauri/src/photo.rs` | Read JPEG → base64, save for print |
| Printing | `src-tauri/src/printer.rs` | lpr/CUPS print |
| Full detail | `./CLAUDE.md` | Architecture, deps, build steps |

## CODE MAP

| Symbol | Type | File | Role |
|--------|------|------|------|
| `App.vue` | Component | `src/App.vue` | Root — view router |
| `useCaptureStore` | Pinia | `src/stores/capture.ts` | Photos + print state |
| `useCameraStore` | Pinia | `src/stores/camera.ts` | Device connection |
| `lib::main()` | Tauri | `src-tauri/src/lib.rs` | Command registration |
| `camera::list_devices` | Command | `src-tauri/src/camera.rs` | Device discovery |
| `camera::verify_device` | Fn | `src-tauri/src/camera.rs` | Pair verification |
| `stream::start` | Command | `src-tauri/src/stream.rs` | MJPEG stream start |
| `stream::capture_photo` | Command | `src-tauri/src/stream.rs` | Photo capture |
| `photo::read_jpeg` | Command | `src-tauri/src/photo.rs` | JPEG → base64 |
| `printer::print` | Command | `src-tauri/src/printer.rs` | lpr print |
| `diagnose_connection` | Command | `src-tauri/src/lib.rs` | Diagnostics |

## CONVENTIONS

- **Async Rust**: All commands use `tokio::task::block_in_place()` to keep WebView unblocked
- **Executable lookup**: `utils::find_executable()` — app data dir first, then PATH
- **Swift helper**: Spawned child process, stdin-driven, stdout-heartbeat for liveness
- **No FFmpeg**: Photos are JPEG, read directly without transcoding
- **Canvas compositing**: Frontend Canvas renders print preview, backend only handles file I/O + print

## ANTI-PATTERNS (FORBIDDEN)

- `as any`, `@ts-ignore`, `@ts-expect-error` anywhere in TypeScript
- `unwrap()` or `panic!` in production Rust code
- Blocking WebView thread with synchronous I/O
- Hardcoded paths (use `utils::find_executable` / Tauri `resolve_path`)
