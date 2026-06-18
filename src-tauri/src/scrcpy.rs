// scrcpy 进程管理与窗口嵌入
// 通过 Rust 后端管理 scrcpy 进程，实现手机画面到大屏的低延迟实时投屏

use std::process::{Child, Command};
use std::sync::{Mutex, OnceLock};
use tokio::task;

static SCRCPY_PROCESS: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

fn get_process_lock() -> &'static Mutex<Option<Child>> {
    SCRCPY_PROCESS.get_or_init(|| Mutex::new(None))
}

/// 启动 scrcpy 投屏
pub async fn start(device_id: String) -> Result<String, String> {
    let lock = get_process_lock();
    
    // 先关闭旧进程
    {
        if let Ok(mut guard) = lock.lock() {
            if let Some(mut old_child) = guard.take() {
                let _ = old_child.kill();
                let _ = old_child.wait();
            }
        }
    }

    // 查找 scrcpy
    let scrcpy_path = crate::utils::find_executable("scrcpy")
        .ok_or_else(|| "scrcpy not found in PATH or app directory".to_string())?;

    // 查找 ADB
    let _adb_path = crate::utils::find_executable("adb")
        .ok_or_else(|| "adb not found in PATH or app directory".to_string())?;

    // 构建 scrcpy 参数
    let mut args = vec![
        "-s".to_string(), device_id.clone(),
        "--stay-awake".to_string(),
        "--disable-screensaver".to_string(),
        "--no-audio".to_string(),
        "--video-codec=h264".to_string(),
        "--max-fps=60".to_string(),
        "--video-bit-rate=8M".to_string(),
        "--max-size=1920".to_string(),
        "--window-borderless".to_string(),
        "--always-on-top".to_string(),
    ];

    // 使用唯一标题便于窗口查找
    args.push("--window-title".to_string());
    args.push(format!("photobooth_scrcpy_{}", std::process::id()));

    let child = task::block_in_place(|| -> Result<std::process::Child, String> {
        Command::new(&scrcpy_path)
            .args(&args)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .map_err(|e| format!("Failed to start scrcpy: {}", e))
    })?;

    {
        let mut guard = lock.lock().map_err(|e| format!("Lock error: {}", e))?;
        *guard = Some(child);
    }

    // Windows 下等待窗口创建
    #[cfg(target_os = "windows")]
    {
        let mut attempts = 0;
        while attempts < 50 {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if is_window_ready() {
                break;
            }
            attempts += 1;
        }
    }

    Ok(format!("scrcpy started for device: {}", device_id))
}

/// 停止 scrcpy
pub async fn stop() -> Result<(), String> {
    let lock = get_process_lock();
    let mut guard = lock.lock().map_err(|e| format!("Lock error: {}", e))?;
    
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
        let _ = child.wait();
    }
    
    Ok(())
}

/// 检查是否在运行
pub fn is_running() -> bool {
    let lock = get_process_lock();
    match lock.lock() {
        Ok(guard) => guard.is_some(),
        Err(_) => false,
    }
}

/// Windows 下检查窗口是否就绪
#[cfg(target_os = "windows")]
fn is_window_ready() -> bool {
    // 通过 EnumWindows 查找是否有 photobooth_scrcpy_ 前缀的窗口
    unsafe {
        let mut found = false;
        let _ = windows::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(std::mem::transmute(enum_windows_proc)),
            &mut found as *mut _ as isize,
        );
        found
    }
}

#[cfg(target_os = "windows")]
extern "system" fn enum_windows_proc(hwnd: isize, param: isize) -> i32 {
    let found = unsafe { &mut *(param as *mut bool) };
    if let Ok(title) = unsafe {
        let mut buf = [0u16; 256];
        let len = windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(
            windows::Win32::Foundation::HWND(hwnd),
            &mut buf,
        );
        String::from_utf16_lossy(&buf[..len as usize])
    } {
        if title.starts_with("photobooth_scrcpy_") {
            *found = true;
        }
    }
    1
}

/// Windows 窗口嵌入：将 scrcpy 窗口嵌入到主窗口
#[cfg(target_os = "windows")]
pub fn embed_scrcpy_window(parent_hwnd: isize, x: i32, y: i32, width: i32, height: i32) -> Result<(), String> {
    unsafe {
        // 找到 scrcpy 窗口句柄
        let mut scrcpy_hwnd = isize::default();
        let mut found = false;
        
        let _ = windows::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(embed_enum_proc),
            &mut EmbedContext {
                parent: parent_hwnd,
                x, y, width, height,
                found_hwnd: &mut scrcpy_hwnd,
                found: &mut found,
            } as *mut _ as isize,
        );
        
        if !found {
            return Err("scrcpy window not found".to_string());
        }
        
        // 修改窗口样式
        let style = windows::Win32::UI::WindowsAndMessaging::GetWindowLongPtrA(
            windows::Win32::Foundation::HWND(scrcpy_hwnd as isize),
            windows::Win32::UI::WindowsAndMessaging::GWL_STYLE,
        );
        let new_style = (style.0 & !(0x80000000 | 0x00C00000 | 0x04000000)) | 0x40000000;
        windows::Win32::UI::WindowsAndMessaging::SetWindowLongPtrA(
            windows::Win32::Foundation::HWND(scrcpy_hwnd as isize),
            windows::Win32::UI::WindowsAndMessaging::GWL_STYLE,
            new_style,
        );
        
        // 设置父窗口
        windows::Win32::UI::WindowsAndMessaging::SetParent(
            windows::Win32::Foundation::HWND(scrcpy_hwnd as isize),
            windows::Win32::Foundation::HWND(parent_hwnd as isize),
        );
        
        // 调整位置
        windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
            windows::Win32::Foundation::HWND(scrcpy_hwnd as isize),
            windows::Win32::UI::WindowsAndMessaging::HWND_TOP,
            x, y, width, height,
            windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE | windows::Win32::UI::WindowsAndMessaging::SWP_SHOWWINDOW,
        );
    }
    
    Ok(())
}

#[cfg(target_os = "windows")]
struct EmbedContext<'a> {
    parent: isize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    found_hwnd: &'a mut isize,
    found: &'a mut bool,
}

#[cfg(target_os = "windows")]
extern "system" fn embed_enum_proc(hwnd: isize, param: isize) -> i32 {
    let ctx = unsafe { &*(param as *const EmbedContext) };
    if let Ok(title) = unsafe {
        let mut buf = [0u16; 256];
        let len = windows::Win32::UI::WindowsAndMessaging::GetWindowTextW(
            windows::Win32::Foundation::HWND(hwnd),
            &mut buf,
        );
        String::from_utf16_lossy(&buf[..len as usize])
    } {
        if title.starts_with("photobooth_scrcpy_") {
            *ctx.found_hwnd = hwnd;
            *ctx.found = true;
        }
    }
    1
}
