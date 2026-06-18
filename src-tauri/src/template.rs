// 模板合成引擎 — JSON 驱动的 Canvas + FFmpeg 双引擎
// 一份 JSON 配置驱动前端 Canvas 预览和后端 FFmpeg 视频合成
// object-fit: cover 双端一致性算法

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SlotConfig {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub rotation: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub image_width: u32,
    pub image_height: u32,
    #[serde(rename = "imgUrl")]
    pub img_url: String,
    pub annotations: Vec<SlotConfig>,
}

/// 模板合成 — 将照片按配置嵌入模板
pub async fn composite(photos: &[String], template_json: &str) -> Result<String, String> {
    let config: TemplateConfig = serde_json::from_str(template_json)
        .map_err(|e| format!("Invalid template JSON: {}", e))?;
    
    if photos.is_empty() {
        return Err("No photos provided for template composition".to_string());
    }
    
    let save_dir = crate::utils::assets_dir();
    crate::utils::ensure_dir(&save_dir).map_err(|e| e.to_string())?;
    
    let output_path = save_dir.join(format!(
        "composite_{}.jpg",
        uuid::Uuid::new_v4()
    ));
    
    // 使用 FFmpeg 进行合成（支持视频模板）
    let ffmpeg = crate::utils::find_executable("ffmpeg")
        .unwrap_or_else(|| "ffmpeg".to_string());
    
    // 构建 FFmpeg 滤镜链
    let mut inputs = Vec::new();
    let mut filter_parts = Vec::new();
    
    // 添加模板背景
    inputs.extend_from_slice(&["-i", &config.img_url]);
    
    // 为每个槽位添加照片
    for (i, photo) in photos.iter().enumerate() {
        let slot = &config.annotations[i % config.annotations.len()];
        let input_idx = i + 1;
        
        // object-fit: cover 等效滤镜
        let cover_filter = format!(
            "[{input_idx}:v]scale='if(gt(iw*{},ih*{}),-1,{}):'if(gt(iw*{},ih*{}),{}, -1)':flags=bilinear,crop={}:{}:(iw-{})/2:(ih-{})/2,setsar=1[v{i}]",
            slot.height, slot.width, slot.width,
            slot.height, slot.width, slot.height,
            slot.width, slot.height,
            slot.width, slot.height,
        );
        filter_parts.push(cover_filter);
        inputs.extend_from_slice(&["-i", photo]);
    }
    
    // 构建叠加滤镜
    let overlay_filters: Vec<String> = config.annotations.iter().enumerate().map(|(i, slot)| {
        format!("[v{}]overlay={}:{}", i, slot.x, slot.y)
    }).collect();
    
    let filter_complex = format!(
        "{};{}",
        filter_parts.join(";"),
        overlay_filters.join("")
    );
    
    let out_str = output_path
        .to_str()
        .ok_or("Invalid path")?
        .to_string();

    let result = tokio::task::block_in_place(|| {
        std::process::Command::new(&ffmpeg)
            .args(&inputs)
            .args(&["-filter_complex", &filter_complex])
            .args(&["-map", "[0:v]"])
            .args(&["-q:v", "2"])
            .args(&["-y"])
            .args(&[&out_str])
            .output()
    });
    
    match result {
        Ok(output) if output.status.success() => {
            Ok(output_path.to_string_lossy().to_string())
        }
        _ => {
            // 如果 FFmpeg 合成失败，返回占位符
            Err("FFmpeg composition failed. Check FFmpeg installation and template path.".to_string())
        }
    }
}

/// object-fit: cover 算法 — 前端 Canvas 使用
/// 保持图片比例填满容器，居中裁剪溢出部分
#[allow(dead_code)]
pub fn calculate_cover_crop(
    img_width: f64,
    img_height: f64,
    container_width: f64,
    container_height: f64,
) -> (f64, f64, f64, f64) {
    let img_ratio = img_width / img_height;
    let container_ratio = container_width / container_height;
    
    let (sx, sy, s_width, s_height) = if img_ratio > container_ratio {
        // 图片更宽 → 裁剪左右，保留上下
        let s_height = img_height;
        let s_width = img_height * container_ratio;
        ((img_width - s_width) / 2.0, 0.0, s_width, s_height)
    } else {
        // 图片更高 → 裁剪上下，保留左右
        let s_width = img_width;
        let s_height = img_width / container_ratio;
        (0.0, (img_height - s_height) / 2.0, s_width, s_height)
    };
    
    (sx, sy, s_width, s_height)
}

/// 获取模板配置的 JSON 字符串
#[allow(dead_code)]
pub fn template_to_json(config: &TemplateConfig) -> Result<String, String> {
    serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize template: {}", e))
}
