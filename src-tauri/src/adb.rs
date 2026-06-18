// ADB 管理器 — 通用 Android 设备兼容
// 支持所有 Android 品牌：android / OPPO / Samsung / Xiaomi / Huawei / OnePlus 等

use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::task;

// ========== 设备信息缓存 ==========

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: String,
    pub model: String,
    pub manufacturer: String,
    pub android_version: String,
    pub brand: String,
}

static DEVICE_INFO_CACHE: std::sync::Mutex<Option<HashMap<String, (DeviceInfo, Instant)>>> =
    std::sync::Mutex::new(None);

fn init_cache() -> std::sync::MutexGuard<'static, Option<HashMap<String, (DeviceInfo, Instant)>>> {
    DEVICE_INFO_CACHE.lock().unwrap_or_else(|e| e.into_inner())
}

const DEVICE_INFO_CACHE_TTL: Duration = Duration::from_secs(30);

// ========== 通用 DCIM 路径 (所有 Android 设备) ==========

const DCIM_PATHS: &[&str] = &[
    "/sdcard/DCIM",
    "/storage/emulated/0/DCIM",
    "/sdcard/DCIM/Camera",
    "/storage/emulated/0/DCIM/Camera",
    "/sdcard/DCIM/androidCamera",
    "/sdcard/DCIM/HuaweiCamera",
    "/sdcard/DCIM/SamsungCamera",
    "/sdcard/DCIM/MiCamera",
    "/sdcard/DCIM/OPPO Camera",
];

/// 查找 ADB 路径
pub fn find_adb_path() -> Result<String, String> {
    crate::utils::find_executable("adb")
        .ok_or_else(|| {
            "ADB not found. Please install Android SDK Platform-Tools.\n\
             macOS: brew install android-platform-tools\n\
             Windows: Download from https://developer.android.com/studio/releases/platform-tools".to_string()
        })
}

/// 获取已连接设备列表
pub async fn get_device_list() -> Result<Vec<String>, String> {
    let adb = find_adb_path()?;

    let output_bytes = task::block_in_place(|| -> Result<Vec<u8>, String> {
        let output = Command::new(&adb)
            .args(["devices", "-l"])
            .output()
            .map_err(|e| format!("Failed to run adb: {}", e))?;
        Ok(output.stdout)
    })?;

    let text = String::from_utf8_lossy(&output_bytes);
    let mut devices = Vec::new();

    for line in text.lines().skip(1) {
        // adb -l 输出示例:
        //   ABC123       device usb:123 product:xxx model:XXX device:XXX
        //   ABC123       unauthorized
        if line.contains("device") && !line.contains("unauthorized") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if !parts.is_empty() {
                devices.push(parts[0].to_string());
            }
        }
    }

    if devices.is_empty() {
        return Err(
            "未检测到 Android 设备\n\n请检查:\n\
             1. 手机通过 USB 连接到电脑\n\
             2. 手机上已开启「开发者选项」和「USB 调试」\n\
             3. 手机上点击「允许 USB 调试」\n\
             4. 尝试更换 USB 数据线或 USB 端口".to_string()
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

    let adb = find_adb_path()?;

    let model = task::block_in_place(|| -> Result<String, String> {
        let output = Command::new(&adb)
            .args(["-s", device_id, "shell", "getprop", "ro.product.model"])
            .output()
            .map_err(|e| format!("Failed to get model: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })?;

    let manufacturer = task::block_in_place(|| -> Result<String, String> {
        let output = Command::new(&adb)
            .args(["-s", device_id, "shell", "getprop", "ro.product.manufacturer"])
            .output()
            .map_err(|e| format!("Failed to get manufacturer: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })?;

    let brand = task::block_in_place(|| -> Result<String, String> {
        let output = Command::new(&adb)
            .args(["-s", device_id, "shell", "getprop", "ro.product.brand"])
            .output()
            .map_err(|e| format!("Failed to get brand: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })?;

    let android_version = task::block_in_place(|| -> Result<String, String> {
        let output = Command::new(&adb)
            .args(["-s", device_id, "shell", "getprop", "ro.build.version.release"])
            .output()
            .map_err(|e| format!("Failed to get Android version: {}", e))?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    })?;

    let info = DeviceInfo {
        id: device_id.to_string(),
        model,
        manufacturer,
        android_version,
        brand,
    };

    // 写入缓存
    {
        let mut cache = init_cache();
        let map = cache.get_or_insert_with(HashMap::new);
        map.insert(device_id.to_string(), (info.clone(), Instant::now()));
    }

    Ok(info)
}

/// 获取设备上的照片列表 (扫描所有 DCIM 路径)
pub async fn get_photo_list(device_id: &str) -> Result<Vec<String>, String> {
    let adb = find_adb_path()?;

    // 使用 find 命令扫描所有 DCIM 路径
    let script = DCIM_PATHS
        .iter()
        .map(|p| format!("find '{}' -maxdepth 2 -type f \\( -name '*.jpg' -o -name '*.jpeg' -o -name '*.png' -o -name '*.heic' -o -name '*.heif' \\) 2>/dev/null", p))
        .collect::<Vec<_>>()
        .join("; ");

    let output_bytes = task::block_in_place(|| -> Result<Vec<u8>, String> {
        Command::new(&adb)
            .args(["-s", device_id, "shell", &script])
            .output()
            .map_err(|e| format!("Failed to get photo list: {}", e))?;
        Ok(Vec::new())
    })?;

    // 回退到简单方案：只扫描 /sdcard/DCIM
    let result = task::block_in_place(|| -> Result<std::process::Output, String> {
        Command::new(&adb)
            .args(["-s", device_id, "shell",
                   "find /sdcard/DCIM -type f \\( -name '*.jpg' -o -name '*.jpeg' -o -name '*.png' -o -name '*.heic' -o -name '*.heif' \\) 2>/dev/null | head -50"])
            .output()
            .map_err(|e| format!("Failed: {}", e))
    })?;

    let text = String::from_utf8_lossy(&result.stdout);
    let photos: Vec<String> = text
        .lines()
        .filter(|l| {
            let l = l.trim();
            !l.is_empty() && (l.ends_with(".jpg") || l.ends_with(".jpeg") ||
                              l.ends_with(".png") || l.ends_with(".heic") || l.ends_with(".heif"))
        })
        .map(|l| l.trim().to_string())
        .collect();

    Ok(photos)
}

/// 获取照片缩略图 (Base64 编码)
pub async fn get_photo_thumbnail(
    filename: &str,
    device_id: &str,
    flip: Option<bool>,
) -> Result<String, String> {
    // 方案 1: 使用 adb exec-out + ffmpeg 提取缩略图
    let flip_filter = if flip.unwrap_or(false) { ",hflip" } else { "" };

    let command_str = format!(
        "adb -s {} exec-out ffmpeg -i {} -frames:v 1 -q:v 2 -f image2pipe -vcodec mjpeg -{}",
        device_id,
        filename,
        flip_filter.trim_start_matches(',')
    );

    let result = task::block_in_place(|| -> Result<Vec<u8>, String> {
        let output = Command::new("sh")
            .arg("-c")
            .arg(&command_str)
            .output()
            .map_err(|e| format!("Failed to extract thumbnail: {}", e))?;
        Ok(output.stdout)
    })?;

    if !result.is_empty() {
        return Ok(crate::utils::base64_encode(&result));
    }

    // 方案 2: 回退到 adb pull 到本地 + ffmpeg 转换
    let temp_dir = crate::utils::app_data_dir();
    let temp_file = format!("{}/thumb_{}.jpg", temp_dir.to_string_lossy(), uuid::Uuid::new_v4());

    let pull_result = task::block_in_place(|| -> Result<std::process::Output, String> {
        Command::new("adb")
            .args(["-s", device_id, "pull", filename, &temp_file])
            .output()
            .map_err(|e| format!("Failed to pull photo: {}", e))
    })?;

    if pull_result.status.success() {
        // 本地用 ffmpeg 提取缩略图
        let thumb_cmd = format!(
            "ffmpeg -i {} -frames:v 1 -q:v 2 -f image2pipe -vcodec mjpeg pipe:1",
            temp_file
        );

        let thumb_result = task::block_in_place(|| -> Result<Vec<u8>, String> {
            let output = Command::new("sh")
                .arg("-c")
                .arg(&thumb_cmd)
                .output()
                .map_err(|e| format!("Failed to extract thumbnail locally: {}", e))?;
            Ok(output.stdout)
        })?;

        if !thumb_result.is_empty() {
            // 清理临时文件
            let _ = std::fs::remove_file(&temp_file);
            return Ok(crate::utils::base64_encode(&thumb_result));
        }
    }

    Err("无法提取照片缩略图。请检查照片路径和设备连接。".to_string())
}

/// 触发拍照 (通用方案 — 所有 Android 设备)
pub async fn trigger_capture(device_id: &str) -> Result<String, String> {
    let adb = crate::utils::find_executable("adb")
        .ok_or("adb not found")?;

    // 发送音量键事件 (大多数相机 app 响应)
    let result = task::block_in_place(|| -> Result<std::process::Output, String> {
        Command::new(&adb)
            .args(["-s", device_id, "shell", "input", "keyevent", "27"])
            .output()
            .map_err(|e| format!("Failed to trigger capture: {}", e))
    })?;

    if !result.status.success() {
        return Err("无法触发拍照。请确保相机应用在前台运行。".to_string());
    }

    // 等待照片生成
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // 获取最新照片
    let photos = get_photo_list(device_id).await?;
    if let Some(latest) = photos.first() {
        Ok(latest.clone())
    } else {
        Err("未检测到新照片。请检查相机权限和存储权限。".to_string())
    }
}

/// 获取设备名称用于显示
pub async fn get_device_display_name(device_id: &str) -> Result<String, String> {
    let info = get_device_info(device_id).await?;
    if !info.model.is_empty() {
        Ok(format!("{} {} (Android {})", info.manufacturer, info.model, info.android_version))
    } else {
        Ok(format!("Android 设备 ({})", device_id))
    }
}

/// 合并获取照片状态 (文件名 + 数量)
pub async fn get_photo_status(device_id: &str) -> Result<(String, usize), String> {
    let photos = get_photo_list(device_id).await?;
    if let Some(latest) = photos.first() {
        Ok((latest.clone(), photos.len()))
    } else {
        Ok((String::new(), 0))
    }
}
