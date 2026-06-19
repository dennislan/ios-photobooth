// 相机流生命周期管理
// 启动 Swift 辅助进程 (ios_camera_stream) 实现 MJPEG 预览推流 + 拍照捕获
// 与 Swift 进程通过 stdin/stdout 通信：
//   - stdin: 发送 "capture" 触发拍照，"quit" 退出
//   - stdout: 接收 "STREAM_READY" / "CAPTURE_SAVED:<path>" / "ERR:<msg>" 信号

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tokio::task;

/// 相机流状态
struct StreamState {
    child: Option<Child>,
    stdin: Option<ChildStdin>,
    /// 最近一次拍照保存的文件路径
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

/// MJPEG 预览端口（与 Swift helper 中一致）
const MJPEG_PORT: u16 = 27183;

/// 启动相机预览流
///
/// 流程：验证设备 → 查找 helper → 释放端口 → 启动子进程 → 等待 STREAM_READY
pub async fn start(device_id: String) -> Result<String, String> {
    // 关闭可能残留的旧进程并释放端口
    stop_internal();
    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1) 验证设备可达且已配对
    let device_name = task::block_in_place(|| crate::camera::verify_device(&device_id))?;

    // 2) 查找 Swift 辅助工具
    let helper_path = find_helper().ok_or_else(|| {
        "未找到相机辅助工具 ios_camera_stream。\n\
         请先构建：cd src-tauri/ios_camera_stream && swift build -c release\n\
         并复制到 src-tauri/resources/ios_camera_stream"
            .to_string()
    })?;

    // 3) 预清理端口占用
    free_port();

    // 4) 重置状态
    {
        let mut guard = state().lock().map_err(|e| format!("锁错误: {}", e))?;
        guard.signal = None;
        guard.exited = false;
        guard.last_capture = None;
    }

    // 5) 启动 helper 子进程
    let mut child = task::block_in_place(|| -> Result<Child, String> {
        Command::new(&helper_path)
            .arg(&device_id)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("启动相机辅助进程失败: {}", e))
    })?;

    let stdin = child.stdin.take().ok_or("无法获取 helper stdin")?;
    let stdout = child.stdout.take().ok_or("无法获取 helper stdout")?;
    let stderr = child.stderr.take().ok_or("无法获取 helper stderr")?;

    {
        let mut guard = state().lock().map_err(|e| format!("锁错误: {}", e))?;
        guard.stdin = Some(stdin);
    }

    // 后台线程：读取 stdout 捕获信号
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().flatten() {
            let line = line.trim().to_string();
            if line.starts_with("STREAM_READY")
                || line.starts_with("CAPTURE_SAVED:")
                || line.starts_with("ERR:")
            {
                if let Ok(mut guard) = state().lock() {
                    if let Some(path) = line.strip_prefix("CAPTURE_SAVED:") {
                        guard.last_capture = Some(path.to_string());
                    } else {
                        guard.signal = Some(line);
                    }
                }
            }
        }
        // stdout 关闭 = 进程退出
        if let Ok(mut guard) = state().lock() {
            guard.exited = true;
        }
    });

    // 后台线程：读取 stderr 仅记录日志
    std::thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().flatten() {
            log::debug!("[ios_camera_stream] {}", line);
        }
    });

    // 存储 child 句柄
    {
        let mut guard = state().lock().map_err(|e| format!("锁错误: {}", e))?;
        guard.child = Some(child);
    }

    // 6) 等待 STREAM_READY 信号（最多 8 秒）
    let deadline = Instant::now() + Duration::from_secs(8);
    loop {
        {
            let mut guard = state().lock().map_err(|e| format!("锁错误: {}", e))?;
            if guard.exited {
                let sig = guard.signal.clone().unwrap_or_default();
                return Err(format!(
                    "相机辅助进程意外退出。{}\n\
                     可能原因：\n\
                     1. macOS 未授予相机权限（系统设置 > 隐私与安全 > 相机）\n\
                     2. iPhone 未作为 Continuity Camera 连接\n\
                     3. 摄像头被其他应用占用",
                    if sig.is_empty() { String::new() } else { format!(" 信号: {}", sig) }
                ));
            }
            if let Some(sig) = &guard.signal {
                if let Some(msg) = sig.strip_prefix("ERR:") {
                    return Err(format!("相机启动失败: {}", msg));
                }
                if sig == "STREAM_READY" {
                    guard.signal = None;
                    return Ok(format!(
                        "已连接到 iPhone: {} ({})",
                        device_name,
                        &device_id[..device_id.len().min(8)]
                    ));
                }
            }
        }
        if Instant::now() > deadline {
            return Err(
                "连接超时（8 秒）。\n\n请检查：\n\
                 1. 系统是否弹出了相机权限请求，请点击「允许」\n\
                 2. 系统设置 > 隐私与安全 > 相机，确认大头贴已勾选\n\
                 3. iPhone 是否解锁并亮屏，USB 连接稳定"
                    .to_string(),
            );
        }
        tokio::time::sleep(Duration::from_millis(150)).await;
    }
}

/// 触发拍照：向 helper stdin 发送 "capture"，等待返回文件路径
pub async fn capture_photo() -> Result<String, String> {
    // 清空上次拍照记录
    {
        let mut guard = state().lock().map_err(|e| format!("锁错误: {}", e))?;
        guard.last_capture = None;
    }

    // 发送 capture 指令
    {
        let mut guard = state().lock().map_err(|e| format!("锁错误: {}", e))?;
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
        stdin.flush().map_err(|e| format!("flush 失败: {}", e))?;
    }

    // 轮询等待拍照结果（最多 6 秒）
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        {
            let mut guard = state().lock().map_err(|e| format!("锁错误: {}", e))?;
            if guard.exited {
                return Err("拍照过程中相机辅助进程退出，请重新连接".to_string());
            }
            if let Some(path) = guard.last_capture.take() {
                return Ok(path);
            }
        }
        if Instant::now() > deadline {
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

/// 检查相机流是否正在运行
pub fn is_running() -> bool {
    let guard = match state().lock() {
        Ok(g) => g,
        Err(_) => return false,
    };
    guard.child.is_some() && !guard.exited
}

/// MJPEG 预览端口（供诊断命令使用）
#[allow(dead_code)]
pub fn mjpeg_port() -> u16 {
    MJPEG_PORT
}

/// 查找相机辅助工具（供诊断命令复用）
pub fn find_helper() -> Option<String> {
    // 1. 应用数据目录
    let candidate = crate::utils::app_data_dir().join("ios_camera_stream");
    if candidate.exists() {
        return Some(candidate.to_string_lossy().to_string());
    }

    // 2. .app 包内 Resources 目录（打包后路径）
    if let Ok(exe) = std::env::current_exe() {
        if let Some(contents) = exe.parent().and_then(|p| p.parent()) {
            let bundled = contents
                .join("Resources")
                .join("resources")
                .join("ios_camera_stream");
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

    // 4. 系统 PATH
    crate::utils::find_executable("ios_camera_stream")
}

// ── 内部函数 ──

fn stop_internal() {
    let mut guard = match state().lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    // 通知 helper 优雅退出
    if let Some(stdin) = guard.stdin.as_mut() {
        let _ = stdin.write_all(b"quit\n");
        let _ = stdin.flush();
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

/// 释放 MJPEG 端口占用（杀掉残留的旧 helper 进程）
fn free_port() {
    let _ = task::block_in_place(|| {
        Command::new("sh")
            .arg("-c")
            .arg(format!("lsof -ti tcp:{} 2>/dev/null | xargs kill -9 2>/dev/null; true", MJPEG_PORT))
            .output()
    });
}
