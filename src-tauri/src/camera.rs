// iPhone 设备发现与验证
// 通过 libimobiledevice (idevice_id / ideviceinfo) 发现并验证已连接的 iPhone

use std::collections::HashMap;
use std::process::Command;
use std::time::{Duration, Instant};
use tokio::task;

/// 设备信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub ios_version: String,
    pub model: String,
}

// ── 设备信息缓存（TTL 30 秒，避免频繁调用 ideviceinfo）──

static DEVICE_INFO_CACHE: std::sync::Mutex<Option<HashMap<String, (DeviceInfo, Instant)>>> =
    std::sync::Mutex::new(None);

const CACHE_TTL: Duration = Duration::from_secs(30);

fn read_cache(device_id: &str) -> Option<DeviceInfo> {
    let guard = DEVICE_INFO_CACHE.lock().ok()?;
    let map = guard.as_ref()?;
    let (info, created) = map.get(device_id)?;
    if created.elapsed() < CACHE_TTL {
        Some(info.clone())
    } else {
        None
    }
}

fn write_cache(device_id: &str, info: DeviceInfo) {
    if let Ok(mut guard) = DEVICE_INFO_CACHE.lock() {
        let map = guard.get_or_insert_with(HashMap::new);
        map.insert(device_id.to_string(), (info, Instant::now()));
    }
}

// ── 工具查找 ──

/// 查找 libimobiledevice 工具（idevice_id / ideviceinfo）
fn find_tool(name: &str) -> Result<String, String> {
    crate::utils::find_executable(name).ok_or_else(|| {
        format!(
            "未找到 {}。请安装 libimobiledevice：\n  brew install libimobiledevice",
            name
        )
    })
}

// ── 公共 API ──

/// 枚举通过 USB 连接的 iPhone 设备列表
pub async fn list_devices() -> Result<Vec<String>, String> {
    let idevice_id = find_tool("idevice_id")?;

    let output = task::block_in_place(|| -> Result<std::process::Output, String> {
        Command::new(&idevice_id)
            .arg("-l")
            .output()
            .map_err(|e| format!("运行 idevice_id 失败: {}", e))
    })?;

    if !output.status.success() {
        return Err("无法枚举设备，libimobiledevice 可能未正确安装。".to_string());
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let devices: Vec<String> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();

    if devices.is_empty() {
        return Err(
            "未检测到 iPhone 设备。\n\n请检查：\n\
             1. iPhone 通过 USB 数据线连接到电脑\n\
             2. iPhone 已解锁并亮屏\n\
             3. iPhone 上已点击「信任此电脑」\n\
             4. 已安装 libimobiledevice：brew install libimobiledevice"
                .to_string(),
        );
    }

    Ok(devices)
}

/// 验证设备是否真正可达且已配对（信任此电脑）
/// 返回设备名称，失败时返回具体的错误原因
pub fn verify_device(device_id: &str) -> Result<String, String> {
    let idevice_id = find_tool("idevice_id")?;

    // 1) USB 枚举层确认设备在列表中
    let list_output = Command::new(&idevice_id)
        .arg("-l")
        .output()
        .map_err(|e| format!("运行 idevice_id 失败: {}", e))?;

    if !list_output.status.success() {
        return Err("无法枚举设备，libimobiledevice 可能未正确安装。".to_string());
    }

    let list_text = String::from_utf8_lossy(&list_output.stdout);
    let connected: Vec<&str> = list_text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if connected.is_empty() {
        return Err(
            "未检测到 iPhone 设备。\n\n请检查：\n\
             1. iPhone 通过 USB 数据线连接到电脑\n\
             2. iPhone 已解锁并亮屏\n\
             3. 尝试更换 USB 数据线或端口\n\
             4. 确保已安装 libimobiledevice：brew install libimobiledevice"
                .to_string(),
        );
    }

    if !connected.iter().any(|d| *d == device_id) {
        return Err(format!(
            "目标设备 {} 不在已连接列表中。当前连接设备：{}",
            &device_id[..device_id.len().min(8)],
            connected
                .iter()
                .map(|d| &d[..d.len().min(8)])
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // 2) 应用层连接验证配对状态
    let ideviceinfo = find_tool("ideviceinfo")?;
    let info_output = Command::new(&ideviceinfo)
        .args(["-u", device_id, "-k", "DeviceName"])
        .output()
        .map_err(|e| format!("运行 ideviceinfo 失败: {}", e))?;

    if !info_output.status.success() {
        let stderr = String::from_utf8_lossy(&info_output.stderr);
        if stderr.contains("lockdownd") || stderr.contains("pair") {
            return Err(
                "iPhone 未信任此电脑。\n\n请在 iPhone 屏幕上点击「信任此电脑」并输入密码，然后重试。"
                    .to_string(),
            );
        }
        if stderr.contains("No device") || stderr.contains("not found") {
            return Err("设备 USB 连接已断开，请重新插拔数据线。".to_string());
        }
        return Err(format!("无法连接到设备：{}", stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&info_output.stdout)
        .trim()
        .to_string())
}

/// 获取设备详细信息（带缓存）
pub async fn device_info(device_id: &str) -> Result<DeviceInfo, String> {
    if let Some(cached) = read_cache(device_id) {
        return Ok(cached);
    }

    let ideviceinfo = find_tool("ideviceinfo")?;

    let run_query = |key: &str| -> Result<String, String> {
        let output = task::block_in_place(|| {
            Command::new(&ideviceinfo)
                .args(["-u", device_id, "-k", key])
                .output()
                .map_err(|e| format!("获取 {} 失败: {}", key, e))
        })?;
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    };

    let name = run_query("DeviceName")?;
    let ios_version = run_query("ProductVersion")?;
    let product_type = run_query("ProductType")?;

    let info = DeviceInfo {
        id: device_id.to_string(),
        name: if name.is_empty() {
            "iPhone".to_string()
        } else {
            name
        },
        ios_version: if ios_version.is_empty() {
            "Unknown".to_string()
        } else {
            ios_version
        },
        model: product_type,
    };

    write_cache(device_id, info.clone());
    Ok(info)
}
