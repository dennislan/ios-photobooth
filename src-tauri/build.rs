use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    tauri_build::build();

    // 自动构建 Swift 相机辅助工具，确保 cargo run / tauri dev 时 helper 始终最新。
    // 仅在 macOS 上执行（AVFoundation 是 macOS 专有框架）。
    #[cfg(target_os = "macos")]
    build_swift_helper();
}

#[cfg(target_os = "macos")]
fn build_swift_helper() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string()));
    let swift_dir = manifest_dir.join("ios_camera_stream");
    let resources_dir = manifest_dir.join("resources");
    let helper_bin = resources_dir.join("ios_camera_stream");

    // Swift 源码目录不存在则跳过（可能尚未 clone 完整）
    if !swift_dir.join("Package.swift").exists() {
        println!("cargo:warning=Swift helper 源码未找到，跳过构建: {}", swift_dir.display());
        return;
    }

    // 检查 swift 编译器是否可用
    let swift_available = Command::new("swift")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !swift_available {
        println!("cargo:warning=swift 编译器不可用，跳过 helper 构建。请安装 Xcode CLT: xcode-select --install");
        return;
    }

    // 增量构建：如果 helper 二进制比所有 Swift 源文件新，则跳过
    if let Some(true) = is_helper_up_to_date(&swift_dir, &helper_bin) {
        return;
    }

    // 确保 resources 目录存在
    if let Err(e) = std::fs::create_dir_all(&resources_dir) {
        println!("cargo:warning=无法创建 resources 目录: {}", e);
        return;
    }

    // 判断 debug / release
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let (swift_config, swift_output_rel) = if profile == "release" {
        ("release", ".build/release/ios_camera_stream")
    } else {
        ("debug", ".build/debug/ios_camera_stream")
    };

    println!("cargo:warning=[build.rs] 构建 Swift helper ({} mode)...", swift_config);

    let output = Command::new("swift")
        .arg("build")
        .arg("-c")
        .arg(swift_config)
        .current_dir(&swift_dir)
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let swift_output = swift_dir.join(swift_output_rel);
            if !swift_output.exists() {
                println!("cargo:warning=Swift 构建产物未找到: {}", swift_output.display());
                return;
            }
            if let Err(e) = std::fs::copy(&swift_output, &helper_bin) {
                println!("cargo:warning=复制 helper 二进制失败: {}", e);
                return;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = std::fs::metadata(&helper_bin) {
                    let mut perms = meta.permissions();
                    perms.set_mode(0o755);
                    let _ = std::fs::set_permissions(&helper_bin, perms);
                }
            }
            println!("cargo:warning=[build.rs] Swift helper 构建完成 ✓");
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            println!("cargo:warning=[build.rs] Swift 构建失败:");
            for line in stderr.lines().take(20) {
                println!("cargo:warning=  {}", line);
            }
        }
        Err(e) => {
            println!("cargo:warning=[build.rs] 无法执行 swift build: {}", e);
        }
    }

    // 告知 cargo：Swift 源文件变化时重新运行 build.rs
    println!("cargo:rerun-if-changed=ios_camera_stream/Sources/main.swift");
    println!("cargo:rerun-if-changed=ios_camera_stream/Package.swift");
}

#[cfg(target_os = "macos")]
fn is_helper_up_to_date(swift_dir: &PathBuf, helper_bin: &PathBuf) -> Option<bool> {
    let helper_mtime = std::fs::metadata(helper_bin).ok()?.modified().ok()?;
    let sources_dir = swift_dir.join("Sources");
    let package_swift = swift_dir.join("Package.swift");

    // 检查 Package.swift
    if let Ok(meta) = std::fs::metadata(&package_swift) {
        if let Ok(mtime) = meta.modified() {
            if mtime > helper_mtime {
                return Some(false);
            }
        }
    }

    // 检查 Sources 目录下所有 .swift 文件
    if let Ok(entries) = std::fs::read_dir(&sources_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("swift") {
                if let Ok(meta) = std::fs::metadata(&path) {
                    if let Ok(mtime) = meta.modified() {
                        if mtime > helper_mtime {
                            return Some(false);
                        }
                    }
                }
            }
        }
    }

    Some(true)
}
