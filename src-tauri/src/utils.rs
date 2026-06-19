// 共享工具函数 — 路径解析、目录管理、Base64 编码、可执行文件查找

use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// 应用数据目录：~/Library/Application Support/photobooth/
pub fn app_data_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("photobooth")
}

/// 临时工作目录：/tmp/photobooth/
pub fn temp_dir() -> PathBuf {
    PathBuf::from("/tmp/photobooth")
}

/// 确保目录存在，不存在则递归创建
pub fn ensure_dir(path: &std::path::Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 确保文件具有可执行权限。
///
/// Tauri 在打包资源到 `.app/Contents/Resources` 时可能丢失执行位，
/// 导致 `Command::new(helper).spawn()` 报 "Permission denied (os error 13)"。
/// 此函数检测任一可执行位（user/group/other）是否设置，未设置则为三者补齐。
#[cfg(unix)]
pub fn ensure_executable(path: &std::path::Path) -> std::io::Result<()> {
    let mut perms = std::fs::metadata(path)?.permissions();
    if perms.mode() & 0o111 == 0 {
        perms.set_mode(perms.mode() | 0o111);
        std::fs::set_permissions(path, perms)?;
        log::info!("已为 {} 补充可执行权限", path.display());
    }
    Ok(())
}

/// Base64 编码字节数组
pub fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// 解码 Base64 字符串（自动去除 data URL 前缀）
pub fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    let cleaned = data
        .trim_start_matches("data:image/jpeg;base64,")
        .trim_start_matches("data:image/png;base64,")
        .trim_start_matches("data:image/jpg;base64,");
    base64::engine::general_purpose::STANDARD
        .decode(cleaned)
        .map_err(|e| format!("Base64 解码失败: {}", e))
}

/// macOS Homebrew 常见安装路径
const HOMEBREW_PATHS: &[&str] = &[
    "/opt/homebrew/bin", // Apple Silicon
    "/usr/local/bin",    // Intel
];

/// 应用启动时补全 PATH 环境变量
///
/// 从 Finder/Spotlight 启动的 GUI 应用，PATH 仅含 /usr/bin:/bin，
/// 不包含 Homebrew 路径，导致 idevice_id/ffmpeg 等工具无法被发现。
/// 此函数在应用启动时将 Homebrew 路径注入 PATH。
pub fn setup_environment() {
    if let Some(current_path) = std::env::var_os("PATH") {
        let current = current_path.to_string_lossy().to_string();
        let mut parts: Vec<&str> = current.split(':').collect();
        for brew in HOMEBREW_PATHS {
            if !parts.contains(brew) {
                parts.push(brew);
            }
        }
        let new_path = parts.join(":");
        std::env::set_var("PATH", &new_path);
        log::info!("PATH 已补全 Homebrew 路径");
    }
}

/// 查找可执行文件：优先应用数据目录 → Homebrew 路径 → 系统 PATH
pub fn find_executable(name: &str) -> Option<String> {
    // 1. 应用数据目录（允许随应用分发外部工具）
    let candidate = app_data_dir().join(name);
    if candidate.exists() {
        return Some(candidate.to_string_lossy().to_string());
    }

    // 2. Homebrew 常见路径（GUI 应用 PATH 可能不含这些路径）
    for brew in HOMEBREW_PATHS {
        let path = std::path::Path::new(brew).join(name);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }

    // 3. 系统 PATH
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in path_var.to_string_lossy().split(':') {
            let path = std::path::Path::new(dir).join(name);
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    None
}
