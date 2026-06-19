// Restart helper — spawns a background bash script that survives the app quitting.
// The script waits for the old process to exit, cleans up the .app.old bundle,
// and launches the new .app bundle.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Spawn a restart helper that will:
/// 1. Wait for the current app process to fully exit
/// 2. Remove the .app.old bundle
/// 3. Launch the new .app bundle
///
/// This returns immediately. The spawned bash process survives the parent app
/// quitting because it's spawned with nohup + disown, making it independent.
pub fn spawn_restart_helper(app_name: &str) -> Result<(), String> {
    let helper_script = get_helper_script_path();
    let script_content = format!(
        r#"#!/bin/bash
set -e

APP_NAME="{}"
APPS_DIR="$(dirname "$APP_NAME")"
BUNDLE_NAME="$(basename "$APP_NAME")"
OLD_BUNDLE="${{BUNDLE_NAME%.app}}.app.old"

# Wait for the old photobooth process to fully exit
MAX_WAIT=60
ELAPSED=0
while [ $ELAPSED -lt $MAX_WAIT ]; do
    # Check if any photobooth process is still running
    if ! pgrep -f "photobooth\.app/Contents/MacOS/photobooth" > /dev/null 2>&1; then
        break
    fi
    sleep 0.5
    ELAPSED=$((ELAPSED + 1))
done

if [ $ELAPSED -ge $MAX_WAIT ]; then
    echo "[update] Timeout waiting for old process to exit" >&2
fi

# Grace period for file descriptors to be released
sleep 1

# Remove old bundle
if [ -d "$APPS_DIR/$OLD_BUNDLE" ]; then
    rm -rf "$APPS_DIR/$OLD_BUNDLE"
    echo "[update] Cleaned up old bundle"
fi

# Launch the new app
open -a "$APP_NAME"
echo "[update] Launched new version of $APP_NAME"
"#,
        app_name
    );

    // Write the script
    if let Some(parent) = helper_script.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create helper script dir: {}", e))?;
    }
    fs::write(&helper_script, &script_content).map_err(|e| format!("Failed to write helper script: {}", e))?;

    // Make executable
    Command::new("chmod")
        .args(["+x", &helper_script.to_string_lossy()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .map_err(|e| format!("Failed to chmod helper script: {}", e))?;

    // Spawn as independent session leader (survives parent exit)
    Command::new("bash")
        .arg(&helper_script)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn restart helper: {}", e))?;

    log::info!("Restart helper spawned for {}", app_name);
    Ok(())
}

/// Path to the restart helper script, stored in the app's local data dir.
fn get_helper_script_path() -> PathBuf {
    let base = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("photobooth").join("restart-helper.sh")
}
