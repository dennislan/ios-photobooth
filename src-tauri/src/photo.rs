// 照片文件 I/O — 读取 JPEG 文件转 Base64、保存 Base64 图片到临时文件供打印

use std::path::Path;

/// 读取 JPEG 照片文件并返回 Base64 编码字符串
///
/// Swift helper 拍照保存的即为 JPEG 格式，无需 ffmpeg 转码，
/// 直接读取文件字节并 Base64 编码即可供前端 <img> 显示。
pub fn read_jpeg(path: &str) -> Result<String, String> {
    if !Path::new(path).exists() {
        return Err(format!("照片文件不存在: {}", path));
    }
    let bytes = std::fs::read(path).map_err(|e| format!("读取照片失败: {}", e))?;
    Ok(crate::utils::base64_encode(&bytes))
}

/// 将 Base64 编码的图片数据保存到临时文件，返回文件路径
///
/// 前端 Canvas 渲染的合成图（用于打印）通过此命令落地为文件，
/// 随后交给 printer 模块调用 lpr 打印。
pub fn save_base64_image(data: &str, ext: &str) -> Result<String, String> {
    let bytes = crate::utils::base64_decode(data)?;

    let dir = crate::utils::temp_dir();
    crate::utils::ensure_dir(&dir).map_err(|e| e.to_string())?;

    let safe_ext = if ext.eq_ignore_ascii_case("png") { "png" } else { "jpg" };
    let path = dir.join(format!("print_{}.{}", uuid::Uuid::new_v4(), safe_ext));
    std::fs::write(&path, &bytes).map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(path.to_string_lossy().to_string())
}
