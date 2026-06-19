// Tauri 2.0 入口 — 注册核心命令
// 职责：iPhone 相机预览/拍照 + 照片选择打印

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod camera;
mod photo;
mod printer;
mod stream;
mod utils;

// ── 设备发现 ──

#[tauri::command]
async fn discover_devices() -> Result<Vec<String>, String> {
    camera::list_devices().await
}

#[tauri::command]
async fn get_device_info(device_id: String) -> Result<camera::DeviceInfo, String> {
    camera::device_info(&device_id).await
}

// ── 相机流控制 ──

#[tauri::command]
async fn start_camera(device_id: String) -> Result<String, String> {
    stream::start(device_id).await
}

#[tauri::command]
async fn stop_camera() -> Result<(), String> {
    stream::stop().await
}

#[tauri::command]
async fn is_camera_active() -> Result<bool, String> {
    Ok(stream::is_running())
}

#[tauri::command]
async fn capture_photo(device_id: String) -> Result<String, String> {
    // 验证设备可达且已配对
    let _ = tokio::task::block_in_place(|| camera::verify_device(&device_id))?;
    // 通过 stdin 发送 capture 指令，等待真实文件路径
    stream::capture_photo().await
}

// ── 照片处理 ──

/// 读取 JPEG 照片文件，返回 Base64 编码（供前端显示）
#[tauri::command]
async fn read_photo(filename: String) -> Result<String, String> {
    tokio::task::block_in_place(|| photo::read_jpeg(&filename))
}

/// 将 Base64 图片数据保存到临时文件（供打印使用）
#[tauri::command]
async fn save_print_image(data: String, ext: String) -> Result<String, String> {
    tokio::task::block_in_place(|| photo::save_base64_image(&data, &ext))
}

// ── 打印 ──

#[tauri::command]
async fn print_photo(
    image_path: String,
    paper_size: String,
    color_mode: String,
    copies: Option<u32>,
) -> Result<bool, String> {
    tokio::task::block_in_place(|| {
        printer::print(&image_path, &paper_size, &color_mode, copies.unwrap_or(1))
    })
}

// ── 诊断 ──

/// 连接诊断 — 返回环境与设备状态报告，供前端展示排查指引
#[tauri::command]
async fn diagnose_connection() -> Result<serde_json::Value, String> {
    let idevice_ok = utils::find_executable("idevice_id").is_some();
    let ideviceinfo_ok = utils::find_executable("ideviceinfo").is_some();
    let helper_ok = stream::find_helper().is_some();

    let devices: Vec<String> = if idevice_ok {
        camera::list_devices().await.unwrap_or_default()
    } else {
        vec![]
    };

    let device_status: Vec<serde_json::Value> = devices
        .iter()
        .map(|d| {
            let pair_result = tokio::task::block_in_place(|| camera::verify_device(d));
            let (paired, name, error) = match pair_result {
                Ok(n) => (true, n, String::new()),
                Err(e) => (false, String::new(), e),
            };
            serde_json::json!({
                "id": d,
                "id_short": d.chars().take(8).collect::<String>(),
                "paired": paired,
                "name": name,
                "error": error,
            })
        })
        .collect();

    let macos_version = tokio::task::block_in_place(|| -> String {
        std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    });

    let major = macos_version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);

    let port_in_use = tokio::task::block_in_place(|| -> bool {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("lsof -ti tcp:{} >/dev/null 2>&1", stream::mjpeg_port()))
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    });

    Ok(serde_json::json!({
        "idevice_id_installed": idevice_ok,
        "ideviceinfo_installed": ideviceinfo_ok,
        "helper_found": helper_ok,
        "macos_version": macos_version,
        "continuity_camera_supported": major >= 13,
        "devices": device_status,
        "port_in_use": port_in_use,
    }))
}

// ── 应用入口 ──

pub fn main() {
    // 补全 PATH（GUI 应用从 Finder 启动时 PATH 不含 Homebrew 路径）
    utils::setup_environment();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            discover_devices,
            get_device_info,
            start_camera,
            stop_camera,
            is_camera_active,
            capture_photo,
            read_photo,
            save_print_image,
            print_photo,
            diagnose_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
