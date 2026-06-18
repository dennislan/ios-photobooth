// FFmpeg 处理器 — HEVC 转码、Live Photo 检测、照片信息获取
// 硬件加速优先，自动降级到软编码

use std::process::Command;
use std::time::Duration;
use tokio::task;

/// 检测照片是否为 Motion Photo / Live Photo
pub async fn is_motion_photo(filename: &str) -> Result<bool, String> {
    if filename.contains("MVIMG") {
        return Ok(true);
    }
    let stem = filename.trim_end_matches(".jpg");
    let mp4_path = format!("{}.mp4", stem);
    Ok(std::path::Path::new(&mp4_path).exists())
}

/// 获取照片信息
pub async fn get_photo_info(filename: &str) -> Result<serde_json::Value, String> {
    let ffmpeg = crate::utils::find_executable("ffmpeg")
        .unwrap_or_else(|| "ffmpeg".to_string());

    let output = task::block_in_place(|| -> Result<std::process::Output, String> {
        Command::new(&ffmpeg)
            .args(&["-v", "quiet", "-print_format", "json", "-show_format", "-show_streams", filename])
            .output()
            .map_err(|e| format!("Failed to run ffprobe: {}", e))
    })?;

    let text = String::from_utf8_lossy(&output.stdout);
    let info: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

    let is_live = is_motion_photo(filename).await.unwrap_or(false);

    Ok(serde_json::json!({
        "filename": filename,
        "is_live_photo": is_live,
        "info": info,
    }))
}

/// HEVC → H.264 转码（硬件加速优先）
pub async fn transcode_hevc(video_path: &str) -> Result<String, String> {
    let ffmpeg = crate::utils::find_executable("ffmpeg")
        .unwrap_or_else(|| "ffmpeg".to_string());

    let encoders: Vec<&str> = if cfg!(target_os = "macos") {
        vec!["h264_videotoolbox", "libx264"]
    } else {
        vec!["h264_nvenc", "h264_qsv", "h264_amf", "libx264"]
    };

    let save_dir = crate::utils::assets_dir();
    crate::utils::ensure_dir(&save_dir).map_err(|e| e.to_string())?;

    let output_path = save_dir.join(format!("live_{}.mp4", uuid::Uuid::new_v4()));
    let out_str = output_path.to_str().ok_or("Invalid output path")?.to_string();

    for encoder in &encoders {
        let is_soft = *encoder == "libx264";

        let args: Vec<&str> = if is_soft {
            vec!["-i", video_path, "-c:v", encoder, "-preset", "ultrafast", "-crf", "23", "-an", "-y", &out_str]
        } else {
            vec!["-i", video_path, "-c:v", encoder, "-b:v", "5M", "-an", "-y", &out_str]
        };

        let output = task::block_in_place(|| {
            Command::new(&ffmpeg).args(&args).output().map_err(|e| format!("FFmpeg failed: {}", e))
        });

        match output {
            Ok(result) if result.status.success() => {
                if output_path.exists() {
                    let file = std::fs::File::open(&output_path).map_err(|e| e.to_string())?;
                    let mut size_after = 0u64;
                    for _ in 0..30 {
                        std::thread::sleep(Duration::from_millis(200));
                        let size_before = size_after;
                        if let Ok(meta) = file.metadata() { size_after = meta.len(); }
                        if size_before == size_after && size_after > 1000 { break; }
                    }
                    return Ok(output_path.to_string_lossy().to_string());
                }
            }
            _ => continue,
        }
    }

    Err("All encoders failed. Check FFmpeg installation.".to_string())
}
