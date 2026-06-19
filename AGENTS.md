# AGENTS.md

**Project:** 大头贴 Photobooth — macOS desktop photobooth app
**Stack:** Tauri 2.0 · Vue 3 · Rust · Swift · FFmpeg
**Updated:** 2026-06-19

## STRUCTURE

```
vivo-photobooth/
├── src/              # Vue 3 frontend (views, components, stores, styles)
├── src-tauri/        # Rust backend + Tauri config + Swift camera helper
│   ├── src/          #   Rust modules (11 files, Tauri commands)
│   └── ios_camera_stream/  # Swift helper (AVCaptureSession → MJPEG)
├── build.sh          # One-click macOS builder
└── index.html        # Vite entry
```

## WHERE TO LOOK

| Task | File | Notes |
|------|------|-------|
| App entry | `src/main.ts` | Creates Vue app, mounts Pinia + Antd |
| Root component | `src/App.vue` | 4-view router switching |
| Camera state | `src/stores/camera.ts` | Connection state, deviceId |
| Capture logic | `src/stores/capture.ts` | Mode, photos list, device state |
| Tauri entry | `src-tauri/src/lib.rs` | Registers all commands, APP_VERSION |
| Device discovery | `src-tauri/src/ios_camera.rs` | idevice_id/ideviceinfo |
| Stream lifecycle | `src-tauri/src/camera_stream.rs` | Swift helper spawn/monitor |
| Template compositing | `src-tauri/src/template.rs` | FFmpeg filter graph generator |
| Update logic | `src-tauri/src/updater.rs` | Semver check + bundle swap |
| Full detail | `./CLAUDE.md` | Architecture, deps, build steps |

## CODE MAP

| Symbol | Type | File | Role |
|--------|------|------|------|
| `App.vue` | Component | `src/App.vue` | Root — view router |
| `useCaptureStore` | Pinia | `src/stores/capture.ts` | Capture state machine |
| `useCameraStore` | Pinia | `src/stores/camera.ts` | Device connection |
| `useTemplateStore` | Pinia | `src/stores/template.ts` | Template config |
| `useUpdateStore` | Pinia | `src/stores/update.ts` | OTA state |
| `lib::main()` | Tauri | `src-tauri/src/lib.rs` | Command registration |
| `discover_iphone` | Command | `src-tauri/src/ios_camera.rs` | Device discovery |
| `start_stream` | Command | `src-tauri/src/camera_stream.rs` | MJPEG stream |
| `capture_photo` | Command | `src-tauri/src/camera_stream.rs` | Photo capture trigger |
| `composite` | Command | `src-tauri/src/template.rs` | FFmpeg compositing |
| `print_photo` | Command | `src-tauri/src/printer.rs` | lpr/cups print |
| `check_update` | Command | `src-tauri/src/updater.rs` | Version check |

## CONVENTIONS

- **Async Rust**: All commands use `tokio::task::block_in_place()` to keep WebView unblocked
- **Executable lookup**: `utils::find_executable()` — app data dir first, then PATH
- **Swift helper**: Spawned child process, stdin-driven, stdout-heartbeat for liveness
- **Template config**: One JSON drives both Canvas preview and FFmpeg compositing
- **OTA**: Semver comparison, in-place bundle swap, nohup survivor script

## ANTI-PATTERNS (FORBIDDEN)

- `as any`, `@ts-ignore`, `@ts-expect-error` anywhere in TypeScript
- `unwrap()` or `panic!` in production Rust code
- Blocking WebView thread with synchronous I/O
- Hardcoded paths (use `utils::find_executable` / Tauri `resolve_path`)
