// 跨平台打印适配
// macOS: lpr 命令直接打印
// Windows: PowerShell + System.Drawing 精细控制

/// 打印图片
pub fn print_image(
    image_path: &str,
    paper_size: &str,
    color_mode: &str,
) -> Result<bool, String> {
    if !std::path::Path::new(image_path).exists() {
        return Err(format!("Image not found: {}", image_path));
    }
    
    #[cfg(target_os = "macos")]
    {
        print_macos(image_path, paper_size, color_mode)
    }
    
    #[cfg(target_os = "windows")]
    {
        print_windows(image_path, paper_size, color_mode)
    }
    
    #[cfg(target_os = "linux")]
    {
        print_cups(image_path, paper_size, color_mode)
    }
}

#[cfg(target_os = "macos")]
fn print_macos(
    image_path: &str,
    _paper_size: &str,
    color_mode: &str,
) -> Result<bool, String> {
    let mut cmd = std::process::Command::new("lpr");
    
    if color_mode == "color" {
        cmd.arg("-o").arg("print-color-mode=color");
    } else {
        cmd.arg("-o").arg("print-color-mode=monochrome");
    }
    
    cmd.arg(image_path);
    
    let result = cmd.output()
        .map_err(|e| format!("Failed to execute lpr: {}", e))?;
    
    if result.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("lpr failed: {}", stderr))
    }
}

#[cfg(target_os = "windows")]
fn print_windows(
    image_path: &str,
    paper_size: &str,
    _color_mode: &str,
) -> Result<bool, String> {
    // 构建 PowerShell 打印脚本
    let paper_setting = match paper_size {
        "6x8" => "\"6\"x\"8\"",
        "A3" => "\"A3\"",
        "A4" => "\"A4\"",
        _ => "\"Auto\"",
    };
    
    let escaped_path = image_path.replace('\\', "\\\\");
    
    let ps_script = format!(
        r#"
Add-Type -AssemblyName System.Drawing
try {{
    $img = [System.Drawing.Image]::FromFile('{path}')
    $paperSize = [System.Drawing.Printing.PaperKind]::{size}
    
    $pd = New-Object System.Drawing.Printing.PrintDocument
    $pd.PrinterSettings.PrinterName = ($pd.PrinterSettings.InstalledPrinters | Select -First 1)
    $pd.DefaultPageSettings.PaperSize = New-Object System.Drawing.Printing.PaperSize($paperSize, {w}, {h})
    
    $pd.Add_PrintPage({{|$sender, $args|
        $gr = $args.Graphics
        $ratio = [math]::Min($args.PageBounds.Width / $img.Width, $args.PageBounds.Height / $img.Height)
        $w = $img.Width * $ratio
        $h = $img.Height * $ratio
        $x = ($args.PageBounds.Width - $w) / 2
        $y = ($args.PageBounds.Height - $h) / 2
        $gr.DrawImage($img, $x, $y, $w, $h)
    }}})
    
    $pd.Print()
    $img.Dispose()
    Write-Output "Printed successfully"
}} catch {{
    Write-Error $_.Exception.Message
    exit 1
}}
"#,
        path = escaped_path,
        size = if paper_size == "A4" { "A4" } else { "Custom" },
        w = if paper_size == "A4" { 2480 } else { 2400 },
        h = if paper_size == "A4" { 3508 } else { 3200 },
    );
    
    let result = std::process::Command::new("powershell")
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &ps_script])
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;
    
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);
    
    if result.status.success() {
        Ok(true)
    } else {
        Err(format!("PowerShell print failed: {}", stderr))
    }
}

#[cfg(target_os = "linux")]
fn print_cups(
    image_path: &str,
    paper_size: &str,
    color_mode: &str,
) -> Result<bool, String> {
    let mut cmd = std::process::Command::new("lp");
    
    match paper_size {
        "A4" => cmd.arg("-o").arg("media=A4"),
        "A3" => cmd.arg("-o").arg("media=A3"),
        _ => {}
    }
    
    match color_mode {
        "color" => cmd.arg("-o").arg("color=true"),
        _ => cmd.arg("-o").arg("color=false"),
    }
    
    cmd.arg(image_path);
    
    let result = cmd.output()
        .map_err(|e| format!("Failed to execute lp: {}", e))?;
    
    if result.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&result.stderr);
        Err(format!("lp failed: {}", stderr))
    }
}
