// macOS 打印 — 通过 CUPS lpr 命令打印图片
// 支持纸张尺寸、色彩模式、份数控制

use std::path::Path;
use std::process::Command;

/// 打印图片
///
/// - `image_path`: 图片文件路径
/// - `paper_size`: 纸张尺寸 ("4x6" | "5x7" | "6x8" | "A4" | "A3")
/// - `color_mode`: 色彩模式 ("color" | "bw")
/// - `copies`: 打印份数
pub fn print(
    image_path: &str,
    paper_size: &str,
    color_mode: &str,
    copies: u32,
) -> Result<bool, String> {
    if !Path::new(image_path).exists() {
        return Err(format!("图片文件不存在: {}", image_path));
    }

    let copies = copies.max(1);
    let mut cmd = Command::new("lpr");

    // 打印份数
    if copies > 1 {
        cmd.arg("-#").arg(copies.to_string());
    }

    // 纸张尺寸 (CUPS media 选项)
    match paper_size {
        "4x6" => { cmd.arg("-o").arg("media=4x6"); }
        "5x7" => { cmd.arg("-o").arg("media=5x7"); }
        "6x8" => { cmd.arg("-o").arg("media=Custom.6x8in"); }
        "A4"  => { cmd.arg("-o").arg("media=A4"); }
        "A3"  => { cmd.arg("-o").arg("media=A3"); }
        _ => {}
    }

    // 色彩模式
    if color_mode == "color" {
        cmd.arg("-o").arg("print-color-mode=color");
    } else {
        cmd.arg("-o").arg("print-color-mode=monochrome");
    }

    // 适应纸张（缩放填满页面，保留比例）
    cmd.arg("-o").arg("fit-to-page");
    cmd.arg(image_path);

    let result = cmd.output().map_err(|e| format!("执行 lpr 失败: {}", e))?;

    if result.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("lpr 打印失败: {}", stderr.trim()))
    }
}
