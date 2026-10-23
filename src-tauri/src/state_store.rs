use std::path::PathBuf;
use tauri::{AppHandle, Manager, Window};

const STATE_FILE: &str = "tracker_state.json";
const STATE_BACKUP: &str = "tracker_state.json.bak";
const STATE_TMP: &str = "tracker_state.json.tmp";
const STATE_MAX_BYTES: usize = 16 * 1024 * 1024;

fn state_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_local_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

#[tauri::command]
pub fn save_state(app: AppHandle, window: Window, data: String) -> Result<(), String> {
    if window.label() != "main" {
        return Err("forbidden".to_string());
    }
    if data.len() > STATE_MAX_BYTES {
        return Err("state too large".to_string());
    }
    serde_json::from_str::<serde_json::Value>(&data).map_err(|_| "invalid state".to_string())?;

    let dir = state_dir(&app)?;
    let path = dir.join(STATE_FILE);
    let tmp = dir.join(STATE_TMP);
    std::fs::write(&tmp, data.as_bytes()).map_err(|e| e.to_string())?;
    if path.exists() {
        let _ = std::fs::copy(&path, dir.join(STATE_BACKUP));
    }
    std::fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

fn read_valid(path: &PathBuf) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if meta.len() as usize > STATE_MAX_BYTES {
        return None;
    }
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str::<serde_json::Value>(&text).ok()?;
    Some(text)
}

#[tauri::command]
pub fn load_state(app: AppHandle) -> Result<Option<String>, String> {
    let dir = state_dir(&app)?;
    Ok(read_valid(&dir.join(STATE_FILE)).or_else(|| read_valid(&dir.join(STATE_BACKUP))))
}
