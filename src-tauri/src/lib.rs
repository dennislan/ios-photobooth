// Tauri 2.0 入口
// 整合 iOS 相机、FFmpeg、模板合成、打印、Deep Link 等模块

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod camera_stream;
mod ios_camera;
mod ffmpeg;
mod template;
mod printer;
mod deep_link;
mod utils;
mod updater;
mod restart;

/// App version — kept in sync with Cargo.toml [package] version.
const APP_VERSION: &str = "1.0.0";

#[tauri::command]
async fn start_camera_stream(device_id: String) -> Result<String, String> {
    camera_stream::start(device_id).await
}

#[tauri::command]
async fn stop_camera_stream() -> Result<(), String> {
    camera_stream::stop().await
}

#[tauri::command]
async fn is_camera_stream_active() -> Result<bool, String> {
    Ok(camera_stream::is_running())
}

#[tauri::command]
async fn get_device_list() -> Result<Vec<String>, String> {
    ios_camera::get_device_list().await
}

#[tauri::command]
async fn get_photo_thumbnail(
    filename: String,
    flip: Option<bool>,
) -> Result<String, String> {
    ios_camera::get_photo_thumbnail(&filename, flip).await
}

#[tauri::command]
async fn take_photo(device_id: String) -> Result<String, String> {
    // 验证设备可达
    let _ = ios_camera::get_device_display_name(&device_id).await?;
    // 通过 camera_stream 的 stdin 发送 capture 命令，等待 Swift 进程写入并返回真实路径
    camera_stream::capture_photo().await
}

#[tauri::command]
async fn check_live_photo(filename: String) -> Result<bool, String> {
    ffmpeg::is_motion_photo(&filename).await
}

#[tauri::command]
async fn get_photo_info(filename: String) -> Result<serde_json::Value, String> {
    ffmpeg::get_photo_info(&filename).await
}

#[tauri::command]
async fn composite_template(
    photos: Vec<String>,
    template_json: String,
) -> Result<String, String> {
    template::composite(&photos, &template_json).await
}

#[tauri::command]
async fn transcode_hevc(video_path: String) -> Result<String, String> {
    ffmpeg::transcode_hevc(&video_path).await
}

#[tauri::command]
async fn print_image(
    image_path: String,
    paper_size: String,
    color_mode: String,
    copies: Option<u32>,
) -> Result<bool, String> {
    printer::print_image(&image_path, &paper_size, &color_mode, copies.unwrap_or(1))
}

#[tauri::command]
async fn find_idevice_path() -> Result<String, String> {
    ios_camera::find_idevice_id_path()
}

/// 将 base64 编码的图片数据保存到临时文件，返回文件路径
/// 用于前端 Canvas 渲染结果保存后送打印
#[tauri::command]
async fn save_temp_image(data: String, ext: String) -> Result<String, String> {
    use base64::Engine;
    let data = data
        .trim_start_matches("data:image/jpeg;base64,")
        .trim_start_matches("data:image/png;base64,")
        .trim_start_matches("data:image/jpg;base64,");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("Invalid base64: {}", e))?;

    let dir = std::path::Path::new("/tmp/photobooth");
    crate::utils::ensure_dir(dir).map_err(|e| e.to_string())?;

    let safe_ext = if ext == "png" { "png" } else { "jpg" };
    let path = dir.join(format!("output_{}.{}", uuid::Uuid::new_v4(), safe_ext));
    std::fs::write(&path, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(path.to_string_lossy().to_string())
}

/// 将 base64 图片数据写入指定路径 (用于下载保存)
#[tauri::command]
async fn write_image_file(path: String, data: String) -> Result<(), String> {
    use base64::Engine;
    let data = data
        .trim_start_matches("data:image/jpeg;base64,")
        .trim_start_matches("data:image/png;base64,")
        .trim_start_matches("data:image/jpg;base64,");
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("Invalid base64: {}", e))?;
    std::fs::write(&path, &bytes).map_err(|e| format!("Failed to write file: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn get_device_info(device_id: String) -> Result<serde_json::Value, String> {
    let info = ios_camera::get_device_info(&device_id).await?;
    Ok(serde_json::json!({
        "id": info.id,
        "model": info.model,
        "manufacturer": info.manufacturer,
        "ios_version": info.ios_version,
        "brand": info.brand,
    }))
}

/// Check for updates from the configured update endpoint.
#[tauri::command]
async fn check_for_updates(endpoint: String) -> Result<serde_json::Value, String> {
    let current_version = APP_VERSION;
    let manifest = updater::check_for_updates(&endpoint, current_version).await?;

    match manifest {
        Some(m) => Ok(serde_json::json!({
            "available": true,
            "current_version": current_version,
            "latest_version": m.version,
            "changelog": m.changelog,
            "mandatory": m.mandatory,
            "download_url": m.download_url,
        })),
        None => Ok(serde_json::json!({
            "available": false,
            "current_version": current_version,
        })),
    }
}

/// Apply an update: download the zip, extract, replace the .app bundle,
/// spawn the restart helper, then schedule the app to quit.
#[tauri::command]
async fn apply_update(
    endpoint: String,
    app_bundle_path: String,
) -> Result<(), String> {
    let current_version = APP_VERSION;
    let manifest = updater::check_for_updates(&endpoint, current_version).await?;

    let manifest = manifest.ok_or_else(|| "No update available".to_string())?;

    let staging_dir = updater::get_app_data_dir().join("updates");

    let zip_path = updater::download_update(&manifest, &staging_dir).await?;
    restart::spawn_restart_helper(&app_bundle_path)?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    updater::apply_update(&zip_path, &app_bundle_path)?;

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        log::info!("Applying update: quitting process");
        std::process::exit(0);
    });

    Ok(())
}

#[tauri::command]
fn get_app_version() -> String {
    APP_VERSION.to_string()
}

#[tauri::command]
fn parse_deep_link(url: String) -> Result<serde_json::Value, String> {
    match deep_link::parse_deep_link(&url) {
        Some(params) => Ok(serde_json::json!({
            "activity_id": params.activity_id,
            "store_id": params.store_id,
            "template_id": params.template_id,
            "mode": params.mode,
        })),
        None => Ok(serde_json::json!({})),
    }
}

pub fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            start_camera_stream,
            stop_camera_stream,
            is_camera_stream_active,
            get_device_list,
            get_photo_thumbnail,
            get_device_info,
            take_photo,
            check_live_photo,
            get_photo_info,
            composite_template,
            transcode_hevc,
            print_image,
            find_idevice_path,
            save_temp_image,
            write_image_file,
            parse_deep_link,
            check_for_updates,
            apply_update,
            get_app_version,
        ])
        .setup(|app| {
            deep_link::register_scheme(&app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
