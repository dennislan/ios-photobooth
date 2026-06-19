// iOS 相机管理 — 设备发现 + 照片捕获
// 使用 libimobiledevice (idevice_id) 发现连接的 iPhone，通过 AVCaptureSession 拍照

use std::process::Command;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::task;

// ========== 设备信息缓存 ==========

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub model: String,
    pub manufacturer: String,
    pub ios_version: String,
    pub brand: String,
}

static DEVICE_INFO_CACHE: std::sync::Mutex<Option<HashMap<String, (DeviceInfo, Instant)>>> =
    std::sync::Mutex::new(None);

fn init_cache() -> std::sync::MutexGuard<'static, Option<HashMap<String, (DeviceInfo, Instant)>>> {
    DEVICE_INFO_CACHE.lock().unwrap_or_else(|e| e.into_inner())
}

const DEVICE_INFO_CACHE_TTL: Duration = Duration::from_secs(30);

/// 查找 idevice_id 路径
pub fn find_idevice_id_path() -> Result<String, String> {
    crate::utils::find_executable("idevice_id")
        .ok_or_else(|| {
            "idevice_id not found. Please install libimobiledevice.\n\
             macOS: brew install libimobiledevice".to_string()
        })
}

/// 查找 ideviceinfo 路径
pub fn find_imobiledevice_bin(bin: &str) -> Result<String, String> {
    crate::utils::find_executable(bin)
        .ok_or_else(|| {
            format!("{} not found. Please install libimobiledevice.\n\
             macOS: brew install libimobiledevice", bin).to_string()
        })
}

/// 获取已连接 iPhone 设备列表
pub async fn get_device_list() -> Result<Vec<String>, String> {
    let _ = find_idevice_id_path()?;

    let output_bytes = task::block_in_place(|| -> Result<Vec<u8>, String> {
        let output = Command::new("idevice_id")
            .args(["-l"])
            .output()
            .map_err(|e| format!("Failed to run idevice_id: {}", e))?;
        Ok(output.stdout)
    })?;

    let text = String::from_utf8_lossy(&output_bytes);
    let mut devices = Vec::new();

    for line in text.lines() {
        let line = line.trim();
        // idevice_id -l outputs one UUID per line (40-char hex strings)
        if line.len() == 40 && line.chars().all(|c| c.is_ascii_hexdigit()) {
            devices.push(line.to_string());
        }
    }

    if devices.is_empty() {
        return Err(
            "未检测到 iPhone 设备\n\n请检查:\n\
             1. iPhone 通过 USB 连接到电脑\n\
             2. iPhone 上点击「信任此电脑」\n\
             3. 尝试更换 USB 数据线或 USB 端口\n\
             4. 确保已安装 libimobiledevice: brew install libimobiledevice".to_string()
        );
    }

    Ok(devices)
}

/// 获取设备详细信息
pub async fn get_device_info(device_id: &str) -> Result<DeviceInfo, String> {
    // 检查缓存
    {
        let cache = init_cache();
        if let Some(ref map) = *cache {
            if let Some((info, expiry)) = map.get(device_id) {
                if expiry.elapsed() < DEVICE_INFO_CACHE_TTL {
                    return Ok(info.clone());
                }
            }
        }
    }

    let info_tool = find_imobiledevice_bin("ideviceinfo")?;

    // 获取设备名称
    let model = task::block_in_place(|| -> Result<String, String> {
        let output = Command::new(&info_tool)
            .args(["-u", device_id, "-k", "DeviceName"])
            .output()
            .map_err(|e| format!("Failed to get model: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })?;

    // 获取产品版本
    let ios_version = task::block_in_place(|| -> Result<String, String> {
        let output = Command::new(&info_tool)
            .args(["-u", device_id, "-k", "ProductVersion"])
            .output()
            .map_err(|e| format!("Failed to get iOS version: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })?;

    // 获取产品型号
    let product_type = task::block_in_place(|| -> Result<String, String> {
        let output = Command::new(&info_tool)
            .args(["-u", device_id, "-k", "ProductType"])
            .output()
            .map_err(|e| format!("Failed to get product type: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })?;

    let info = DeviceInfo {
        id: device_id.to_string(),
        model: if model.is_empty() { product_type.clone() } else { model },
        manufacturer: "Apple".to_string(),
        ios_version: if ios_version.is_empty() { "Unknown".to_string() } else { ios_version },
        brand: product_type,
    };

    // 写入缓存
    {
        let mut cache = init_cache();
        let map = cache.get_or_insert_with(HashMap::new);
        map.insert(device_id.to_string(), (info.clone(), Instant::now()));
    }

    Ok(info)
}

/// 触发拍照 — 通过 camera_stream 进程发送 capture 指令
// 实际拍照由 camera_stream.rs 管理的 AVCaptureSession 执行
// 注意: lib.rs 的 take_photo 命令直接调用 camera_stream::capture_photo()
#[allow(dead_code)]
pub async fn take_photo(device_id: &str) -> Result<String, String> {
    // 通知 camera_stream 进程执行拍照
    let output = task::block_in_place(|| -> Result<std::process::Output, String> {
        Command::new("idevice_id")
            .args(["-u", device_id, "-t"]) // -t 测试连接
            .output()
            .map_err(|e| format!("Failed to connect to device: {}", e))
    })?;

    if !output.status.success() {
        return Err("无法连接到 iPhone。请在 iPhone 上点击「信任此电脑」。".to_string());
    }

    // 拍照动作由 camera_stream 进程处理
    // 这里返回成功，实际文件路径由 camera_stream 写入后返回
    // 暂时通过 ideviceinfo 确认设备在线
    Ok(format!("/tmp/photobooth/photo_{}.heic", uuid::Uuid::new_v4()))
}

/// 获取照片缩略图 (本地文件)
pub async fn get_photo_thumbnail(
    filename: &str,
    flip: Option<bool>,
) -> Result<String, String> {
    let ffmpeg = find_ffmpeg_path()?;
    let do_flip = flip.unwrap_or(false);

    let result = task::block_in_place(|| -> Result<Vec<u8>, String> {
        let mut cmd = Command::new(&ffmpeg);
        cmd.arg("-i").arg(filename);
        if do_flip {
            cmd.arg("-vf").arg("hflip");
        }
        cmd.arg("-frames:v").arg("1")
            .arg("-q:v").arg("2")
            .arg("-f").arg("image2pipe")
            .arg("-vcodec").arg("mjpeg")
            .arg("pipe:1");
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to extract thumbnail: {}", e))?;
        Ok(output.stdout)
    })?;

    if !result.is_empty() {
        return Ok(crate::utils::base64_encode(&result));
    }

    Err("无法提取照片缩略图。请检查照片路径。".to_string())
}

/// 获取设备名称用于显示
pub async fn get_device_display_name(device_id: &str) -> Result<String, String> {
    let info = get_device_info(device_id).await?;
    if !info.model.is_empty() {
        Ok(format!("{} {} (iOS {})", info.manufacturer, info.model, info.ios_version))
    } else {
        Ok(format!("iPhone ({})", device_id))
    }
}

fn find_ffmpeg_path() -> Result<String, String> {
    crate::utils::find_executable("ffmpeg")
        .ok_or_else(|| {
            "ffmpeg not found. Please install FFmpeg.\n\
             macOS: brew install ffmpeg".to_string()
        })
}
