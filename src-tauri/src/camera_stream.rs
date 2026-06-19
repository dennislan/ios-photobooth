// iOS 相机流管理 — 启动/停止 AVCaptureSession 预览流
// 通过启动 Swift 辅助进程实现 MJPEG 推流到本地 TCP 端口 (27183)
// 保留 stdin 用于发送 capture 指令，读取 stdout 获取 STREAM_READY / CAPTURE_SAVED 信号

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tokio::task;

struct StreamState {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    last_capture: Option<String>,
    /// helper 输出的就绪 / 错误信号
    signal: Option<String>,
    /// helper 是否已退出
    exited: bool,
}

static STREAM: OnceLock<Mutex<StreamState>> = OnceLock::new();

fn state() -> &'static Mutex<StreamState> {
    STREAM.get_or_init(|| {
        Mutex::new(StreamState {
            child: None,
            stdin: None,
            last_capture: None,
            signal: None,
            exited: false,
        })
    })
}

/// 启动相机预览流
pub async fn start(device_id: String) -> Result<String, String> {
    // 先关闭旧进程（并释放端口）
    stop_internal();
    // 给端口释放留点时间
    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1) 验证设备可达 + 已配对（替代无效的 idevice_id -u -t）
    let device_name = task::block_in_place(|| crate::ios_camera::verify_device(&device_id))?;

    // 2) 查找 Swift 辅助工具
    let stream_path = find_camera_stream_helper().ok_or_else(|| {
        "未找到相机辅助工具 ios_camera_stream。\n\
         请先构建: cd src-tauri/ios_camera_stream && swift build -c release".to_string()
    })?;

    // 3) 预清理可能残留的 27183 端口占用
    let _ = free_port_27183();

    // 4) 重置信号状态
    {
        let mut guard = state().lock().map_err(|e| format!("Lock error: {}", e))?;
        guard.signal = None;
        guard.exited = false;
        guard.last_capture = None;
    }

    // 5) 启动 helper，捕获 stdout + stderr
    let mut child = task::block_in_place(|| -> Result<Child, String> {
        Command::new(&stream_path)
            .arg(&device_id)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("启动相机辅助进程失败: {}", e))
    })?;

    let stdin = child
        .stdin
        .take()
        .ok_or("无法获取 helper stdin")?;
    let stdout = child
        .stdout
        .take()
        .ok_or("无法获取 helper stdout")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("无法获取 helper stderr")?;

    {
        let mut guard = state().lock().map_err(|e| format!("Lock error: {}", e))?;
        guard.stdin = Some(stdin);
    }

    // 后台线程读取 stdout：捕获 STREAM_READY / CAPTURE_SAVED 信号（detach 运行）
    let _stdout_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            let trimmed = line.trim().to_string();
            if trimmed.starts_with("STREAM_READY") || trimmed.starts_with("CAPTURE_SAVED:") || trimmed.starts_with("ERR:") {
                if let Ok(mut guard) = state().lock() {
                    if trimmed.starts_with("CAPTURE_SAVED:") {
                        guard.last_capture = Some(trimmed.trim_start_matches("CAPTURE_SAVED:").to_string());
                    } else {
                        guard.signal = Some(trimmed);
                    }
                }
            }
        }
        // stdout 关闭意味着进程退出
        if let Ok(mut guard) = state().lock() {
            guard.exited = true;
        }
    });

    // 后台线程读取 stderr：仅记录（调试用）
    let _stderr_thread = std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            fputs_to_log(&line);
        }
    });

    // 存储 child
    {
        let mut guard = state().lock().map_err(|e| format!("Lock error: {}", e))?;
        guard.child = Some(child);
    }

    // 6) 等待 STREAM_READY 信号（最多 8 秒），期间检测进程是否提前退出
    let start = Instant::now();
    loop {
        {
            let mut guard = state().lock().map_err(|e| format!("Lock error: {}", e))?;
            if guard.exited {
                let sig = guard.signal.clone().unwrap_or_default();
                return Err(format!(
                    "相机辅助进程意外退出。{}\n\
                     可能原因:\n\
                     1. macOS 未授予相机权限（系统设置 > 隐私与安全 > 相机）\n\
                     2. iPhone 未作为 Continuity Camera 连接\n\
                     3. 摄像头被其他应用占用",
                    if sig.is_empty() { String::new() } else { format!(" 信号: {}", sig) }
                ));
            }
            if let Some(sig) = &guard.signal {
                if sig.starts_with("ERR:") {
                    let msg = sig.trim_start_matches("ERR:").to_string();
                    return Err(format!("相机启动失败: {}", msg));
                }
                if sig == "STREAM_READY" {
                    guard.signal = None;
                    return Ok(format!(
                        "已连接到 iPhone: {} ({})",
                        device_name,
                        device_id.chars().take(8).collect::<String>()
                    ));
                }
            }
        }
        if start.elapsed() > Duration::from_secs(8) {
            // 超时：进程还在但没就绪，可能是相机权限弹窗未响应
            return Err(
                "连接超时（8 秒）。\n\n请检查:\n\
                 1. 系统是否弹出了相机权限请求，请点击「允许」\n\
                 2. 系统设置 > 隐私与安全 > 相机，确认大头贴已勾选\n\
                 3. iPhone 是否解锁并亮屏，USB 连接稳定".to_string()
            );
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

fn fputs_to_log(line: &str) {
    log::debug!("[ios_camera_stream] {}", line);
}

/// 尝试释放 27183 端口（杀掉占用该端口的旧 helper 进程）
fn free_port_27183() -> Result<(), String> {
    let out = task::block_in_place(|| -> Result<std::process::Output, String> {
        Command::new("sh")
            .arg("-c")
            .arg("lsof -ti tcp:27183 2>/dev/null | xargs kill -9 2>/dev/null; true")
            .output()
            .map_err(|e| format!("free port failed: {}", e))
    })?;
    let _ = out;
    Ok(())
}

/// 触发拍照 — 向 Swift 进程 stdin 发送 capture 指令，等待返回文件路径
pub async fn capture_photo() -> Result<String, String> {
    // 清空上次拍照记录
    {
        let mut guard = state().lock().map_err(|e| format!("Lock error: {}", e))?;
        guard.last_capture = None;
    }

    // 发送 capture 指令
    {
        let mut guard = state().lock().map_err(|e| format!("Lock error: {}", e))?;
        if guard.exited {
            return Err("相机预览流已断开，请重新连接设备".to_string());
        }
        let stdin = guard
            .stdin
            .as_mut()
            .ok_or("相机预览流未运行，请先连接设备")?;
        stdin
            .write_all(b"capture\n")
            .map_err(|e| format!("发送拍照指令失败: {}", e))?;
        stdin
            .flush()
            .map_err(|e| format!("flush 失败: {}", e))?;
    }

    // 轮询等待拍照结果（最多 6 秒）
    let start = Instant::now();
    loop {
        {
            let mut guard = state().lock().map_err(|e| format!("Lock error: {}", e))?;
            if guard.exited {
                return Err("拍照过程中相机辅助进程退出，请重新连接".to_string());
            }
            if let Some(path) = guard.last_capture.take() {
                return Ok(path);
            }
        }
        if start.elapsed() > Duration::from_secs(6) {
            return Err("拍照超时，请确认相机预览画面正常后重试".to_string());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// 停止相机预览流
pub async fn stop() -> Result<(), String> {
    stop_internal();
    Ok(())
}

fn stop_internal() {
    let mut guard = match state().lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    // 关闭 stdin
    if guard.stdin.is_some() {
        if let Some(stdin) = guard.stdin.as_mut() {
            let _ = stdin.write_all(b"quit\n");
            let _ = stdin.flush();
        }
    }
    guard.stdin = None;
    // 杀掉子进程
    if let Some(mut child) = guard.child.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    guard.last_capture = None;
    guard.signal = None;
}

/// 检查相机流是否运行
pub fn is_running() -> bool {
    let guard = match state().lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    guard.child.is_some() && !guard.exited
}

/// 查找相机流辅助工具（pub 供 diagnose_connection 复用）
pub fn find_camera_stream_helper() -> Option<String> {
    // 1. 从应用数据目录查找
    let app_dir = crate::utils::app_data_dir();
    let candidate = app_dir.join("ios_camera_stream");
    if candidate.exists() {
        return Some(candidate.to_string_lossy().to_string());
    }

    // 2. 从 .app 包内 Resources 目录查找 (Tauri 打包后: Contents/Resources/resources/ios_camera_stream)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(contents) = exe.parent().and_then(|p| p.parent()) {
            let bundled = contents.join("Resources").join("resources").join("ios_camera_stream");
            if bundled.exists() {
                return Some(bundled.to_string_lossy().to_string());
            }
            let direct = contents.join("Resources").join("ios_camera_stream");
            if direct.exists() {
                return Some(direct.to_string_lossy().to_string());
            }
        }
    }

    // 3. 开发时从 src-tauri/resources 查找
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dev_resources) = exe
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(|p| p.join("src-tauri").join("resources").join("ios_camera_stream"))
        {
            if dev_resources.exists() {
                return Some(dev_resources.to_string_lossy().to_string());
            }
        }
    }

    // 4. 从 PATH 查找
    crate::utils::find_executable("ios_camera_stream")
}
