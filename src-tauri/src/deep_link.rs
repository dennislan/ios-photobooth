// Deep Link 协议与单实例保护
// URL Scheme: photobooth://callback?{params}

/// 从 Deep Link URL 解析参数
pub fn parse_deep_link(url: &str) -> Option<DeepLinkParams> {
    if !url.starts_with("photobooth://") {
        return None;
    }

    let query = &url["photobooth://".len()..];
    let mut params = DeepLinkParams::default();

    for pair in query.split('&') {
        let kv: Vec<&str> = pair.splitn(2, '=').collect();
        if kv.len() == 2 {
            let value = url_decode(kv[1]);
            match kv[0] {
                "activity_id" => params.activity_id = Some(value),
                "store_id" => params.store_id = Some(value),
                "template_id" => params.template_id = Some(value),
                "mode" => params.mode = Some(value),
                _ => {}
            }
        }
    }

    Some(params)
}

#[derive(Debug, Default, Clone)]
pub struct DeepLinkParams {
    pub activity_id: Option<String>,
    pub store_id: Option<String>,
    pub template_id: Option<String>,
    pub mode: Option<String>,
}

fn url_decode(s: &str) -> String {
    s.replace("%20", " ")
        .replace("%26", "&")
        .replace("%3D", "=")
        .replace("%3F", "?")
        .replace("%25", "%")
        .replace("%2B", "+")
}

/// 注册 photobooth:// URL Scheme
#[cfg(target_os = "macos")]
pub fn register_scheme(_app: &tauri::AppHandle) -> Result<(), String> {
    println!("URL scheme registered via Info.plist on macOS");
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn register_scheme(_app: &tauri::AppHandle) -> Result<(), String> {
    println!("URL scheme registration handled by installer on Windows");
    Ok(())
}
