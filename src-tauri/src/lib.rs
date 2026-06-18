// Tauri 2.0 入口
// 整合 scrcpy、ADB、FFmpeg、模板合成、打印、Deep Link 等模块

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod scrcpy;
mod adb;
mod ffmpeg;
mod template;
mod printer;
mod deep_link;
mod utils;

#[tauri::command]
async fn start_scrcpy(device_id: String) -> Result<String, String> {
    scrcpy::start(device_id).await
}

#[tauri::command]
async fn stop_scrcpy() -> Result<(), String> {
    scrcpy::stop().await
}

#[tauri::command]
async fn is_scrcpy_running() -> Result<bool, String> {
    Ok(scrcpy::is_running())
}

#[tauri::command]
async fn get_device_list() -> Result<Vec<String>, String> {
    adb::get_device_list().await
}

#[tauri::command]
async fn get_photo_list(device_id: String) -> Result<Vec<String>, String> {
    adb::get_photo_list(&device_id).await
}

#[tauri::command]
async fn get_photo_thumbnail(
    filename: String,
    device_id: String,
    flip: Option<bool>,
) -> Result<String, String> {
    adb::get_photo_thumbnail(&filename, &device_id, flip).await
}

#[tauri::command]
async fn take_photo(device_id: String) -> Result<String, String> {
    adb::trigger_capture(&device_id).await
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
) -> Result<bool, String> {
    printer::print_image(&image_path, &paper_size, &color_mode)
}

#[tauri::command]
async fn find_adb_path() -> Result<String, String> {
    adb::find_adb_path()
}

#[tauri::command]
async fn get_device_info(device_id: String) -> Result<serde_json::Value, String> {
    let info = adb::get_device_info(&device_id).await?;
    Ok(serde_json::json!({
        "id": info.id,
        "model": info.model,
        "manufacturer": info.manufacturer,
        "android_version": info.android_version,
        "brand": info.brand,
    }))
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
            start_scrcpy,
            stop_scrcpy,
            is_scrcpy_running,
            get_device_list,
            get_photo_list,
            get_photo_thumbnail,
            get_device_info,
            take_photo,
            check_live_photo,
            get_photo_info,
            composite_template,
            transcode_hevc,
            print_image,
            find_adb_path,
            parse_deep_link,
        ])
        .setup(|app| {
            deep_link::register_scheme(&app.handle())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
