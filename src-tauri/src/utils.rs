// 工具函数

use std::path::PathBuf;

/// 获取应用数据目录
pub fn app_data_dir() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("photobooth")
}

/// 获取素材目录
pub fn assets_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        app_data_dir()
    } else {
        dirs::desktop_dir().unwrap_or_else(|| PathBuf::from("."))
    };
    base.join("photobooth-assets")
}

/// 确保目录存在
pub fn ensure_dir(path: &std::path::Path) -> std::io::Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// Base64 编码
pub fn base64_encode(data: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
}

/// 获取文件扩展名
fn exe_ext() -> &'static str {
    if cfg!(target_os = "windows") { ".exe" } else { "" }
}

/// 查找可执行文件（优先从应用目录查找，其次从 PATH）
pub fn find_executable(name: &str) -> Option<String> {
    let ext = exe_ext();
    
    // 1. 从应用数据目录查找
    let app_dir = app_data_dir();
    let candidate = app_dir.join(format!("{}{}", name, ext));
    if candidate.exists() {
        return Some(candidate.to_string_lossy().to_string());
    }

    // 2. 从 PATH 查找
    if let Some(path_var) = std::env::var_os("PATH") {
        let sep = if cfg!(target_os = "windows") { ';' } else { ':' };
        for dir in path_var.to_string_lossy().split(sep) {
            let path = std::path::Path::new(dir).join(format!("{}{}", name, ext));
            if path.exists() {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }

    None
}

/// 条件编译：Windows 下隐藏控制台窗口
#[cfg(target_os = "windows")]
pub fn create_hidden_command(program: &str) -> std::process::Command {
    use std::process::Command;
    let mut cmd = Command::new(program);
    cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    cmd
}

/// 条件编译：非 Windows 下正常创建命令
#[allow(dead_code)]
#[cfg(not(target_os = "windows"))]
pub fn create_hidden_command(program: &str) -> std::process::Command {
    std::process::Command::new(program)
}
