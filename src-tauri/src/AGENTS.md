# AGENTS.md — src-tauri/src/

**Role:** 11 Rust modules backing all Tauri 2.0 commands + app lifecycle

## FILE MAP

| File | Role |
|------|------|
| `main.rs` | Binary entry — calls `lib::main()` |
| `lib.rs` | Tauri setup — registers all commands, plugins, `APP_VERSION` |
| `ios_camera.rs` | iPhone discovery via `idevice_id` / `ideviceinfo` with caching |
| `camera_stream.rs` | Swift helper child process lifecycle (spawn/stdin/stop/heartbeat) |
| `ffmpeg.rs` | Live Photo detection, `ffprobe`, HEVC→H.264 transcode |
| `template.rs` | JSON-driven FFmpeg filter graph generator for compositing |
| `printer.rs` | Cross-platform print: macOS `lpr`, Windows PowerShell, Linux `lp` |
| `deep_link.rs` | `photobooth://` URL scheme parsing |
| `updater.rs` | OTA: fetch manifest, semver compare, download zip, bundle swap |
| `restart.rs` | `nohup` survivor script — waits for exit, cleans old .app, relaunches |
| `utils.rs` | Shared: `find_executable()`, path resolution, base64, cache dirs |

## COMMAND REGISTRATION (lib.rs pattern)

```rust
// lib.rs registers every command via .invoke_handler()
// Commands are async, run block_in_place to keep WebView responsive
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            discover_iphone,
            start_stream,
            capture_photo,
            composite,
            print_photo,
            check_update,
            apply_update,
            // ...
        ])
        .run(tauri::generate_context!())
}
```

## KEY DEPENDENCIES

```
lib.rs ─── registers ─── ios_camera.rs, camera_stream.rs, ffmpeg.rs,
                         template.rs, printer.rs, deep_link.rs,
                         updater.rs, restart.rs
                │
           utils.rs (shared helpers — used by most modules)
```

## CONVENTIONS

- **`tokio::task::block_in_place()`** wraps all blocking I/O in Tauri commands
- **`utils::find_executable()`** — app data dir first (`~/Library/Application Support/photobooth/`), then `PATH`
- **Swift helper** — spawned child; stdin for commands (`capture\n`), stdout heartbeat
- **No `unwrap()` / `panic!`** in production code — use `thiserror` + `Result`
- **External binary deps** — `idevice_id`, `ffmpeg`, `ios_camera_stream` — not bundled in build

## ANTI-PATTERNS

- No blocking WebView thread (always `block_in_place`)
- No hardcoded paths — use `utils::find_executable` / Tauri `resolve_path`
- No `unwrap()` in production code paths
- No shelling out with raw strings — use `Command::new(...)` with arg array
