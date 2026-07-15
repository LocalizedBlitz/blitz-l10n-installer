mod config;
mod instance;
mod localization;
mod process;
mod version;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use tauri::Emitter;
use tauri::Manager;

#[derive(Serialize, Deserialize)]
struct UpdateInfo {
    version: String,
    path: String,
}

// TODO: Update to Gitee raw URL once repo is mirrored
const LBI_METADATA_URL: &str = "https://raw.githubusercontent.com/LocalizedBlitz/blitz-l10n-installer/main/metadata.json";

// ── Instance detection ──

#[tauri::command]
fn scan_instances() -> Vec<InstanceInfo> {
    instance::find_all_instances()
        .into_iter()
        .map(|(path, type_code)| InstanceInfo { path, type_code })
        .collect()
}

#[derive(Serialize)]
struct InstanceInfo {
    path: String,
    #[serde(rename = "type")]
    type_code: String,
}

#[tauri::command]
fn validate_instance_path(path: String) -> bool {
    instance::validate_instance_path(PathBuf::from(&path).as_path())
}

// ── Version ──

#[tauri::command]
fn get_exe_version(path: String) -> Option<String> {
    version::get_exe_version(&path)
}

// ── Process ──

#[tauri::command]
fn is_app_running(install_path: String) -> bool {
    process::is_app_running(&install_path)
}

#[tauri::command]
fn force_kill_app(install_path: String) -> Result<(), String> {
    process::force_kill_app(&install_path)
}

#[tauri::command]
fn launch_app(path: String) -> Result<(), String> {
    let exe_path = PathBuf::from(&path);
    let work_dir = exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    Command::new(&path)
        .current_dir(&work_dir)
        .spawn()
        .map_err(|e| format!("Failed to launch application: {}", e))?;
    Ok(())
}

// ── Localization (stubs) ──

#[tauri::command]
async fn fetch_blitz_metadata() -> Result<Vec<localization::BlitzMetadataItem>, String> {
    let app_config = config::load_config().unwrap_or_default();
    let proxy_url = config::build_proxy_url(&app_config.proxy);
    localization::fetch_blitz_metadata(proxy_url.as_deref()).await
}

#[tauri::command]
fn read_cached_blitz_metadata() -> Option<Vec<localization::BlitzMetadataItem>> {
    localization::read_cached_blitz_metadata()
}

#[tauri::command]
async fn download_and_install_blitz(
    app: tauri::AppHandle,
    blitz_path: String,
    lang_id: String,
) -> Result<String, String> {
    let app_config = config::load_config().unwrap_or_default();
    let proxy_url = config::build_proxy_url(&app_config.proxy);
    localization::install_blitz(
        Some(&app),
        &blitz_path,
        &lang_id,
        proxy_url.as_deref(),
    )
    .await
}

#[tauri::command]
async fn full_uninstall(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        let target_dir = PathBuf::from(&path);
        localization::full_uninstall(&target_dir)
    })
    .await
    .map_err(|e| format!("Uninstall task panicked: {}", e))?
}

#[tauri::command]
fn check_blitz_status(blitz_path: String) -> localization::BlitzStatus {
    let exe_version = version::get_exe_version(&blitz_path).unwrap_or_default();
    localization::check_blitz_status(&blitz_path, &exe_version)
}

// ── Config ──

#[tauri::command]
fn get_app_config() -> Result<config::AppConfig, String> {
    config::load_config()
}

#[tauri::command]
fn save_app_config(app_config: config::AppConfig) -> Result<(), String> {
    config::save_config(&app_config)
}

#[tauri::command]
fn get_data_dir() -> String {
    config::get_data_dir().to_string_lossy().to_string()
}

#[tauri::command]
fn get_cache_size() -> String {
    let bytes = localization::get_cache_size_bytes();
    if bytes >= 1_073_741_824 {
        format!("{:.2} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[tauri::command]
fn clear_cache() -> Result<(), String> {
    localization::clear_all_caches()
}

// ── App info ──

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn version_newer(remote: &str, local: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> {
        v.split('.').filter_map(|s| s.parse().ok()).collect()
    };
    let r = parse(remote);
    let l = parse(local);
    for i in 0..r.len().max(l.len()) {
        let rn = r.get(i).copied().unwrap_or(0);
        let ln = l.get(i).copied().unwrap_or(0);
        if rn > ln {
            return true;
        }
        if rn < ln {
            return false;
        }
    }
    false
}

#[tauri::command]
async fn check_lbi_update() -> Result<Option<UpdateInfo>, String> {
    let app_config = config::load_config().unwrap_or_default();
    let proxy_url = config::build_proxy_url(&app_config.proxy);
    let client = localization::build_client(proxy_url.as_deref())?;
    let response = client
        .get(LBI_METADATA_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch update metadata: {}", e))?;
    let info: UpdateInfo = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse update metadata: {}", e))?;
    let current = env!("CARGO_PKG_VERSION");
    if version_newer(&info.version, current) {
        Ok(Some(info))
    } else {
        Ok(None)
    }
}

// ── Initial language ──

#[tauri::command]
fn resolve_initial_language() -> String {
    let cfg_path = config::config_path();
    if cfg_path.exists() {
        config::load_config()
            .map(|c| c.language)
            .unwrap_or_else(|_| String::from("zh_CN"))
    } else {
        config::system_locale_to_language()
    }
}

// ── Theme ──

#[tauri::command]
fn set_window_theme(app: tauri::AppHandle, theme: String) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_theme(if theme == "dark" {
            Some(tauri::Theme::Dark)
        } else {
            Some(tauri::Theme::Light)
        });
    }
}

// ── Proxy password ──

#[tauri::command]
fn save_proxy_password(password: String) -> Result<(), String> {
    config::save_proxy_password(&password)
}

#[tauri::command]
fn load_proxy_password() -> Result<Option<String>, String> {
    config::load_proxy_password()
}

#[tauri::command]
fn delete_proxy_password() -> Result<(), String> {
    config::delete_proxy_password()
}

// ── Self-update ──

#[derive(Clone, Serialize)]
struct UpdateDownloadProgress {
    percent: u32,
    downloaded_bytes: u64,
    total_bytes: u64,
}

#[tauri::command]
async fn download_and_install_update(app: tauri::AppHandle, download_url: String) -> Result<(), String> {
    use futures_util::StreamExt;
    use std::io::Write;

    let app_config = config::load_config().unwrap_or_default();
    let proxy_url = config::build_proxy_url(&app_config.proxy);
    let client = localization::build_client(proxy_url.as_deref())?;
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;
    let total_bytes = response.content_length().unwrap_or(0);
    let temp_dir = std::env::temp_dir();
    let installer_path = temp_dir.join("LBI_Setup.exe");
    let mut file = std::fs::File::create(&installer_path)
        .map_err(|e| format!("Failed to create installer file: {}", e))?;
    let mut stream = response.bytes_stream();
    let mut downloaded: u64 = 0;
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;
        let percent = if total_bytes > 0 {
            ((downloaded as f64 / total_bytes as f64) * 100.0).min(99.0) as u32
        } else {
            0
        };
        let _ = app.emit("update-download-progress", UpdateDownloadProgress {
            percent,
            downloaded_bytes: downloaded,
            total_bytes: total_bytes.max(downloaded),
        });
    }
    let _ = app.emit("update-download-progress", UpdateDownloadProgress {
        percent: 100,
        downloaded_bytes: downloaded,
        total_bytes: downloaded,
    });
    drop(file);
    std::process::Command::new(&installer_path)
        .spawn()
        .map_err(|e| format!("Failed to launch installer: {}", e))?;
    app.exit(0);
    Ok(())
}

// ── Entry point ──

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if let Ok(cfg) = config::load_config() {
                if let Some(window) = app.get_webview_window("main") {
                    let theme = if cfg.theme == "dark" {
                        Some(tauri::Theme::Dark)
                    } else {
                        Some(tauri::Theme::Light)
                    };
                    let _ = window.set_theme(theme);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_instances,
            validate_instance_path,
            get_exe_version,
            is_app_running,
            force_kill_app,
            launch_app,
            fetch_blitz_metadata,
            read_cached_blitz_metadata,
            download_and_install_blitz,
            full_uninstall,
            check_blitz_status,
            get_app_config,
            save_app_config,
            get_data_dir,
            get_cache_size,
            clear_cache,
            get_app_version,
            resolve_initial_language,
            check_lbi_update,
            set_window_theme,
            save_proxy_password,
            load_proxy_password,
            delete_proxy_password,
            download_and_install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
