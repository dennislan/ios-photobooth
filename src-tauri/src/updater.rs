// Updater module — checks the latest release from GitHub, downloads the new app
// bundle zip, and applies the update by replacing the .app in-place
// (rename-then-copy pattern to avoid macOS file descriptor issues).

use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

/// Internal update manifest (populated from GitHub release data).
#[derive(serde::Deserialize, Debug, Clone)]
#[allow(dead_code)]
pub struct UpdateManifest {
    pub version: String,
    pub download_url: String,
    #[serde(default)]
    pub changelog: String,
    #[serde(default = "default_false")]
    pub mandatory: bool,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub sha256: Option<String>,
}

fn default_false() -> bool {
    false
}

// ── GitHub API types ────────────────────────────────────────────────────────

/// GitHub Release API response (only the fields we care about).
#[derive(serde::Deserialize, Debug)]
struct GitHubRelease {
    tag_name: String,
    #[serde(default)]
    body: String,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<GitHubAsset>,
}

#[derive(serde::Deserialize, Debug)]
struct GitHubAsset {
    name: String,
    #[serde(default)]
    browser_download_url: String,
    #[serde(default)]
    size: Option<u64>,
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Check for updates by querying the GitHub Releases API.
///
/// `repo` — GitHub repository in the form `"owner/repo"` (e.g. `"vivo-ai/photobooth"`).
///
/// Returns `Ok(Some(manifest))` when a newer version exists, `Ok(None)` when
/// already up-to-date, or `Err(message)` on failure (network, parsing, etc.).
pub async fn check_for_updates(repo: &str) -> Result<Option<UpdateManifest>, String> {
    // Allow overriding the repo via environment variable at runtime
    let repo = std::env::var("PHOTOBOOTH_GITHUB_REPO").unwrap_or_else(|_| repo.to_string());
    let current_version = env!("CARGO_PKG_VERSION");

    let api_url = format!("https://api.github.com/repos/{}/releases/latest", repo);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resp = client
        .get(&api_url)
        .header("User-Agent", "photobooth-updater/1.0")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("Failed to reach GitHub API at {}: {}", api_url, e))?;

    if resp.status() == 404 {
        // Repo or release not found
        return Err(format!(
            "GitHub repo '{}' not found or has no releases",
            repo
        ));
    }

    if resp.status() == 403 || resp.status() == 429 {
        // Rate limited
        return Err("GitHub API rate limit exceeded. Try again later or set GITHUB_TOKEN.".to_string());
    }

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned HTTP {}", resp.status()));
    }

    let release: GitHubRelease = resp.json().await.map_err(|e| {
        format!("Failed to parse GitHub release response: {}", e)
    })?;

    // Optionally skip pre-releases (GitHub's /latest already skips them for
    // the default branch, but this is an extra safety check).
    if release.prerelease {
        return Err("Latest release is a pre-release — skipping".to_string());
    }

    // Normalise semver from tag_name: strip leading 'v' or 'V'
    let raw_tag = release.tag_name.trim();
    let version_str = raw_tag
        .strip_prefix('v')
        .or_else(|| raw_tag.strip_prefix('V'))
        .unwrap_or(raw_tag);

    let remote = semver::Version::parse(version_str)
        .map_err(|e| format!("Invalid semver in GitHub tag '{}': {}", raw_tag, e))?;

    // Compare with current version
    let current = semver::Version::parse(current_version)
        .map_err(|e| format!("Invalid current version '{}': {}", current_version, e))?;

    if remote <= current {
        log::info!(
            "App is up to date (current={}, GitHub={})",
            current_version,
            version_str
        );
        return Ok(None);
    }

    // Find the right asset — prefer .app.zip, then any .zip
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.ends_with(".app.zip"))
        .or_else(|| release.assets.iter().find(|a| a.name.ends_with(".zip")))
        .or_else(|| {
            release
                .assets
                .iter()
                .find(|a| a.name.ends_with(".zip"))
        });

    let asset = asset.ok_or_else(|| {
        format!(
            "No matching .zip asset found in release '{}'. Available: {:?}",
            raw_tag,
            release.assets.iter().map(|a| &a.name).collect::<Vec<_>>()
        )
    })?;

    log::info!(
        "Update available: {} -> {} (asset: {}, size: {:?})",
        current_version,
        version_str,
        asset.name,
        asset.size
    );

    Ok(Some(UpdateManifest {
        version: version_str.to_string(),
        download_url: asset.browser_download_url.clone(),
        changelog: release.body,
        mandatory: false,
        size: asset.size,
        sha256: None,
    }))
}

/// Download the update payload to a staging directory.
/// Returns the path to the downloaded .zip file.
pub async fn download_update(
    manifest: &UpdateManifest,
    staging_dir: &Path,
) -> Result<PathBuf, String> {
    // Ensure staging directory exists
    fs::create_dir_all(staging_dir)
        .map_err(|e| format!("Failed to create staging dir: {}", e))?;

    let zip_path = staging_dir.join("update.zip");
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5 min for large downloads
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let resp = client
        .get(&manifest.download_url)
        .send()
        .await
        .map_err(|e| {
            format!(
                "Failed to download update from {}: {}",
                manifest.download_url, e
            )
        })?;

    if !resp.status().is_success() {
        return Err(format!("Download returned HTTP {}", resp.status()));
    }

    let bytes = resp.bytes().await.map_err(|e| {
        format!("Failed to read download body: {}", e)
    })?;

    log::info!("Downloaded {} bytes", bytes.len());

    fs::write(&zip_path, &bytes).map_err(|e| {
        format!("Failed to write update file: {}", e)
    })?;

    Ok(zip_path)
}

/// Find the .app bundle path from the running executable.
/// Resolves: current_exe → Contents/MacOS → Contents → .. → .app
#[allow(dead_code)]
pub fn get_app_bundle_path() -> Result<PathBuf, String> {
    let exe = std::env::current_exe()
        .map_err(|e| format!("Cannot determine executable path: {}", e))?;

    // exe → .../photobooth.app/Contents/MacOS/photobooth
    let bundle = exe
        .parent() // MacOS/
        .and_then(|p| p.parent()) // Contents/
        .and_then(|p| p.parent()) // photobooth.app/
        .map(|p| p.to_path_buf());

    bundle.ok_or_else(|| "Cannot resolve app bundle path".to_string())
}

/// Find a .app bundle inside an extracted directory.
/// Searches one level deep (common for zip extracts).
#[allow(dead_code)]
pub fn find_app_bundle(dir: &Path) -> Option<PathBuf> {
    // Direct children
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.extension().and_then(|e| e.to_str()) == Some("app") {
                return Some(path);
            }
        }
    }

    // One level deep
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_dir()
                            && sub_path.extension().and_then(|e| e.to_str()) == Some("app")
                        {
                            return Some(sub_path);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Apply the update: unzip, rename old bundle, extract new bundle.
/// Must be called BEFORE the process exits — a survivor process should
/// already be running to relaunch the app after quit.
pub fn apply_update(zip_path: &Path, app_bundle_path: &str) -> Result<(), String> {
    let app_name = app_bundle_path
        .strip_prefix("/")
        .map(|s| s)
        .unwrap_or(app_bundle_path);
    // Get just the bundle name (e.g. "photobooth.app")
    let bundle_name = Path::new(app_name)
        .file_name()
        .map(|n| n.to_str().unwrap_or(""))
        .unwrap_or("photobooth.app");

    // Derive the directory containing the .app bundle
    let apps_dir = app_bundle_path
        .rsplit('/')
        .next()
        .map(|d| {
            if d.ends_with(".app") {
                app_bundle_path[..app_bundle_path.len() - d.len()].to_string()
            } else {
                app_bundle_path.to_string()
            }
        })
        .unwrap_or_else(|| "/Applications".to_string());

    let final_bundle = Path::new(&apps_dir).join(&bundle_name);
    let old_bundle = final_bundle.with_extension("app.old");

    // Rename old bundle aside (atomic on same filesystem)
    if final_bundle.exists() {
        if old_bundle.exists() {
            fs::remove_dir_all(&old_bundle)
                .map_err(|e| format!("Failed to remove stale .old bundle: {}", e))?;
        }
        fs::rename(&final_bundle, &old_bundle).map_err(|e| {
            format!(
                "Failed to rename old bundle aside: {}. Make sure the app data directory is writable.",
                e
            )
        })?;
        log::info!("Renamed {} → {}", final_bundle.display(), old_bundle.display());
    }

    // Extract new bundle using system unzip (preserves resource forks)
    let extract_target = Path::new(&apps_dir);

    let output = Command::new("unzip")
        .args(["-o", "-x", "__MACOSX/**", "-d"])
        .arg(extract_target)
        .arg(zip_path)
        .output()
        .map_err(|e| format!("unzip command failed: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("unzip failed: stdout={}, stderr={}", stdout, stderr));
    }

    log::info!("Extracted update to {}", apps_dir);

    // Clean up downloaded zip
    if zip_path.exists() {
        fs::remove_file(zip_path).ok();
    }

    // Clean up old bundle after a short delay (survivor handles this)
    // The survivor script will remove .app.old after verifying the new app launched

    Ok(())
}

/// Get the app data directory for staging updates.
pub fn get_app_data_dir() -> PathBuf {
    let base = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("photobooth")
}
