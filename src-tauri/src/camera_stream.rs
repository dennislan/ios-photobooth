// iOS 相机流管理 — 启动/停止 AVCaptureSession 预览流
// 通过启动 Swift 辅助进程实现 MJPEG 推流到本地 TCP 端口 (27183)
// 保留 stdin 用于发送 capture 指令，读取 stdout 获取拍照后的文件路径

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tokio::task;

struct StreamState {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    last_capture: Option<String>,
}

static STREAM: OnceLock<Mutex<StreamState>> = OnceLock::new();

fn state() -> &'static Mutex<StreamState> {
    STREAM.get_or_init(|| {
        Mutex::new(StreamState {
            child: None,
            stdin: None,
            last_capture: None,
        })
    })
}

/// 启动相机预览流
pub async fn start(device_id: String) -> Result<String, String> {
    // 先关闭旧进程
    stop_internal();

    // 查找 Swift 辅助工具
    let stream_path = find_camera_stream_helper().ok_or_else(|| {
        "iOS camera stream helper not found.\n\
         Please build the Swift helper tool first.\n\
         See: src-tauri/ios_camera_stream/".to_string()
    })?;

    // 验证设备可达
    let verify_output = task::block_in_place(|| -> Result<std::process::Output, String> {
        Command::new("idevice_id")
            .args(["-u", &device_id, "-t"])
            .output()
            .map_err(|e| format!("Failed to verify device: {}", e))
    })?;

    if !verify_output.status.success() {
        return Err(format!(
            "无法连接到 iPhone ({})。请在 iPhone 上点击「信任此电脑」。\n\n\
             设备 ID: {}",
            device_id,
            device_id.chars().take(8).collect::<String>()
        ));
    }

    let mut child = task::block_in_place(|| -> Result<Child, String> {
        Command::new(&stream_path)
            .arg(&device_id)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start camera stream: {}", e))
    })?;

    let stdin = child
        .stdin
        .take()
        .ok_or("Failed to capture stdin of camera stream")?;
    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture stdout of camera stream")?;

    {
        let mut guard = state().lock().map_err(|e| format!("Lock error: {}", e))?;
        guard.stdin = Some(stdin);
        guard.last_capture = None;
    }

    // 后台线程读取 stdout，捕获 CAPTURE_SAVED:<path> 信号
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            if let Some(path) = line.strip_prefix("CAPTURE_SAVED:") {
                let path = path.trim().to_string();
                if let Ok(mut guard) = state().lock() {
                    guard.last_capture = Some(path);
                }
            }
        }
    });

    // 存储 child 进程
    {
        let mut guard = state().lock().map_err(|e| format!("Lock error: {}", e))?;
        guard.child = Some(child);
    }

    // 等待进程就绪
    tokio::time::sleep(Duration::from_millis(1200)).await;

    Ok(format!(
        "Camera stream started for iPhone: {}",
        device_id.chars().take(8).collect::<String>()
    ))
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
        let stdin = guard
            .stdin
            .as_mut()
            .ok_or("相机预览流未运行，请先连接设备")?;
        stdin
            .write_all(b"capture\n")
            .map_err(|e| format!("Failed to send capture command: {}", e))?;
        stdin
            .flush()
            .map_err(|e| format!("Failed to flush capture command: {}", e))?;
    }

    // 轮询等待拍照结果（最多 6 秒）
    let start = Instant::now();
    loop {
        {
            let mut guard = state().lock().map_err(|e| format!("Lock error: {}", e))?;
            if let Some(path) = guard.last_capture.take() {
                return Ok(path);
            }
        }
        if start.elapsed() > Duration::from_secs(6) {
            return Err("拍照超时，请确认相机预览正常后重试".to_string());
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
        // 尝试发送 quit
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
}

/// 检查相机流是否运行
pub fn is_running() -> bool {
    let guard = match state().lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    guard.child.is_some()
}

/// 查找相机流辅助工具
fn find_camera_stream_helper() -> Option<String> {
    // 1. 从应用数据目录查找
    let app_dir = crate::utils::app_data_dir();
    let candidate = app_dir.join("ios_camera_stream");
    if candidate.exists() {
        return Some(candidate.to_string_lossy().to_string());
    }

    // 2. 从 .app 包内 Resources 目录查找 (Tauri 打包后路径: Contents/Resources/resources/ios_camera_stream)
    if let Ok(exe) = std::env::current_exe() {
        if let Some(contents) = exe.parent().and_then(|p| p.parent()) {
            // Tauri bundles resources into a "resources/" subdirectory
            let bundled = contents.join("Resources").join("resources").join("ios_camera_stream");
            if bundled.exists() {
                return Some(bundled.to_string_lossy().to_string());
            }
            // 也检查直接放在 Resources 下的情况
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
