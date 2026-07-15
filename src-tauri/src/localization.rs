use chrono::Local;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::Emitter;

// ── Data structures ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct L10nPackage {
    pub path: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_blitz_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontPackage {
    pub id: String,
    pub name: String,
    pub path: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlitzMetadataItem {
    pub id: String,
    pub name: String,
    pub l10n: L10nPackage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<FontPackage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractManifest {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub supported_game_version: String,
    pub files: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstInfo {
    pub language: String,
    pub date: String,
    pub loc_version: String,
    pub l10n_etag: String,
    pub font_version: String,
    pub font_etag: String,
    pub font_id: String,
    pub installed_game_version: String,
    pub app_version: String,
    pub backups: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlitzStatus {
    pub path: String,
    pub version: String,
    pub loc_installed: bool,
    pub loc_version: String,
    pub font_version: String,
    pub loc_language: String,
    pub is_compatible: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressPayload {
    pub step: String,
    pub percent: u32,
    pub message: String,
    pub message_key: String,
    pub message_params: HashMap<String, String>,
    pub instance: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

const INST_INFO_FILE: &str = "lbi_inst.json";
const METADATA_CACHE_FILE: &str = "blitz_metadata.json";

fn instance_fingerprint(blitz_path: &str) -> String {
    let normalized = blitz_path.to_lowercase().replace('\\', "/");
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();
    hex::encode(&result[..8])
}

fn backup_dir(fingerprint: &str) -> PathBuf {
    crate::config::get_data_dir().join("backups").join(fingerprint)
}

fn cache_dir() -> PathBuf { crate::config::get_data_dir().join("cache") }

fn temp_extract_dir() -> PathBuf { std::env::temp_dir().join("lbi_extract") }

pub fn build_client(proxy_url: Option<&str>) -> Result<reqwest::Client, String> {
    let mut builder = reqwest::Client::builder().timeout(std::time::Duration::from_secs(120));
    if let Some(url) = proxy_url {
        let proxy = reqwest::Proxy::all(url).map_err(|e| format!("Failed to create proxy: {}", e))?;
        builder = builder.proxy(proxy);
    }
    builder.build().map_err(|e| format!("Failed to build HTTP client: {}", e))
}

fn is_version_compatible(supported: &str, actual: &str) -> bool {
    if supported == "*" || supported == actual { return true; }
    if let Some(stripped) = actual.strip_prefix(supported) {
        return stripped.is_empty() || stripped.starts_with('.');
    }
    false
}

// ── Metadata ──

pub async fn fetch_blitz_metadata(_proxy_url: Option<&str>) -> Result<Vec<BlitzMetadataItem>, String> {
    let items = vec![BlitzMetadataItem {
        id: "zh_CN".into(), name: "简体中文".into(),
        l10n: L10nPackage { path: "https://gitee.com/localizedblitz/blitz-l10n-chs/raw/main/l10n.7z".into(), version: String::new(), supported_blitz_version: None },
        font: Some(FontPackage { id: "srcwagon_mc".into(), name: "SrcWagon MainlandCN".into(), path: "https://pub-e75a159ebad9491ea3597f0b48e69823.r2.dev/blitz/fonts/SrcWagon-MainlandCN.7z".into(), version: String::new() }),
    }];
    let _ = write_cached_blitz_metadata(&items);
    Ok(items)
}

pub fn read_cached_blitz_metadata() -> Option<Vec<BlitzMetadataItem>> {
    let path = cache_dir().join(METADATA_CACHE_FILE);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn write_cached_blitz_metadata(items: &[BlitzMetadataItem]) -> Result<(), String> {
    let dir = cache_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create cache dir: {}", e))?;
    let content = serde_json::to_string_pretty(items).map_err(|e| format!("Failed to serialize metadata: {}", e))?;
    std::fs::write(dir.join(METADATA_CACHE_FILE), content).map_err(|e| format!("Failed to write metadata cache: {}", e))?;
    Ok(())
}

fn emit_progress(app: &tauri::AppHandle, instance: &str, step: &str, percent: u32, message_key: &str, downloaded_bytes: u64, total_bytes: u64) {
    let payload = ProgressPayload { step: step.to_string(), percent, message: String::new(), message_key: message_key.to_string(), message_params: HashMap::new(), instance: instance.to_string(), downloaded_bytes, total_bytes };
    let _ = app.emit("install-progress", payload);
}

// ── 7z download + extract ──

async fn download_and_extract_7z(app: Option<&tauri::AppHandle>, instance: &str, url: &str, dest_dir: &Path, proxy_url: Option<&str>, label: &str, if_none_match: Option<&str>) -> Result<String, String> {
    let client = build_client(proxy_url)?;

    // HEAD request to get file size and ETag
    let (total_bytes, etag) = if let Ok(head_resp) = client.head(url).send().await {
        let len = head_resp.content_length().unwrap_or(0);
        let et = head_resp.headers().get("etag").and_then(|v| v.to_str().ok()).unwrap_or("").trim_matches('"').to_string();
        (len, et)
    } else { (0u64, String::new()) };

    // Check cache: cached 7z + matching ETag
    let cache_dir = cache_dir();
    let cached_7z = cache_dir.join(format!("{}.7z", label));
    let cached_etag_path = cache_dir.join(format!("{}.etag", label));
    let cached_etag = std::fs::read_to_string(&cached_etag_path).unwrap_or_default().trim().to_string();

    // If cached file exists and ETag matches → extract from cache
    if cached_7z.exists() && !etag.is_empty() && cached_etag == etag {
        log::info!("{}: cached 7z matches ETag, extracting from cache", label);
        if let Some(app) = app { emit_progress(app, instance, "download", 100, label, 0, 0); }
        if let Some(app) = app { emit_progress(app, instance, "extract", 0, "extracting", 0, 0); }
        sevenz_rust::decompress_file(&cached_7z, dest_dir).map_err(|e| format!("Failed to extract {}: {}", label, e))?;
        if let Some(app) = app { emit_progress(app, instance, "extract", 100, "extracted", 0, 0); }
        return Ok(etag);
    }

    // If stored ETag from lbi_inst matches remote ETag → skip, use cached version from lbi_inst
    if let Some(stored) = if_none_match {
        if !stored.is_empty() && !etag.is_empty() && stored == etag {
            log::info!("{}: ETag unchanged, skipping download", label);
            if let Some(app) = app { emit_progress(app, instance, "download", 100, label, 0, 0); }
            if cached_7z.exists() {
                if let Some(app) = app { emit_progress(app, instance, "extract", 0, "extracting", 0, 0); }
                sevenz_rust::decompress_file(&cached_7z, dest_dir).map_err(|e| format!("Failed to extract {}: {}", label, e))?;
                if let Some(app) = app { emit_progress(app, instance, "extract", 100, "extracted", 0, 0); }
            }
            return Ok(stored.to_string());
        }
    }

    // Need to download — save to cache dir
    let response = client.get(url).send().await.map_err(|e| format!("Failed to download {}: {}", label, e))?;
    let total_bytes = if total_bytes > 0 { total_bytes } else { response.content_length().unwrap_or(0) };
    let etag = if etag.is_empty() { response.headers().get("etag").and_then(|v| v.to_str().ok()).unwrap_or("").trim_matches('"').to_string() } else { etag };
    let mut downloaded: u64 = 0;

    std::fs::create_dir_all(&cache_dir).map_err(|e| format!("Failed to create cache dir: {}", e))?;
    let temp_dl = cache_dir.join(format!("{}.7z.dl", label));
    let mut file = std::fs::File::create(&temp_dl).map_err(|e| format!("Failed to create cache file: {}", e))?;
    let mut stream = response.bytes_stream();
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk).map_err(|e| format!("Write error: {}", e))?;
        downloaded += chunk.len() as u64;
        let pct = if total_bytes > 0 { ((downloaded as f64 / total_bytes as f64) * 100.0).min(99.0) as u32 } else { 0 };
        if let Some(app) = app { emit_progress(app, instance, "download", pct, label, downloaded, total_bytes); }
    }
    drop(file);
    if let Some(app) = app { emit_progress(app, instance, "download", 100, label, downloaded, total_bytes); }

    // Atomically move to cached location and write ETag
    let _ = std::fs::remove_file(&cached_7z);
    std::fs::rename(&temp_dl, &cached_7z).map_err(|e| format!("Failed to save cached 7z: {}", e))?;
    let _ = std::fs::write(&cached_etag_path, &etag);

    // Extract from cache
    if let Some(app) = app { emit_progress(app, instance, "extract", 0, "extracting", 0, 0); }
    std::fs::create_dir_all(dest_dir).map_err(|e| format!("Failed to create extract dir: {}", e))?;
    sevenz_rust::decompress_file(&cached_7z, dest_dir).map_err(|e| format!("Failed to extract {}: {}", label, e))?;
    if let Some(app) = app { emit_progress(app, instance, "extract", 100, "extracted", 0, 0); }

    Ok(etag)
}

// ── File backup ──

fn backup_single_file(data_dir: &Path, relative_path: &str, fingerprint: &str) -> Result<String, String> {
    let src = data_dir.join(relative_path);
    if !src.is_file() { return Ok(String::new()); }
    let bkp_dir = backup_dir(fingerprint);
    std::fs::create_dir_all(&bkp_dir).map_err(|e| format!("Failed to create backup dir: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(relative_path.as_bytes());
    let hash = hex::encode(&hasher.finalize()[..12]);
    let ext = Path::new(relative_path).extension().and_then(|e| e.to_str()).unwrap_or("");
    let backup_name = if ext.is_empty() { format!("{}.bak", hash) } else { format!("{}.{}", hash, ext) };
    let dst = bkp_dir.join(&backup_name);
    std::fs::copy(&src, &dst).map_err(|e| format!("Failed to backup {}: {}", relative_path, e))?;
    Ok(backup_name)
}

fn restore_single_file(data_dir: &Path, relative_path: &str, backup_name: &str, fingerprint: &str) -> Result<(), String> {
    if backup_name.is_empty() { let target = data_dir.join(relative_path); if target.is_file() { let _ = std::fs::remove_file(&target); } return Ok(()); }
    let bkp_dir = backup_dir(fingerprint);
    let src = bkp_dir.join(backup_name);
    let dst = data_dir.join(relative_path);
    if let Some(parent) = dst.parent() { std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir for restore: {}", e))?; }
    std::fs::copy(&src, &dst).map_err(|e| format!("Failed to restore {}: {}", relative_path, e))?;
    Ok(())
}

// ── Install ──

pub async fn install_blitz(app: Option<&tauri::AppHandle>, blitz_path: &str, lang_id: &str, proxy_url: Option<&str>) -> Result<String, String> {
    let blitz_dir = PathBuf::from(blitz_path);
    let data_dir = blitz_dir.join("Data");
    let fingerprint = instance_fingerprint(blitz_path);
    if let Some(app) = app { emit_progress(app, blitz_path, "start", 0, "preparing", 0, 0); }

    let metadata_items = match fetch_blitz_metadata(proxy_url).await {
        Ok(items) if !items.is_empty() => items,
        _ => return Err("No localization metadata available.".to_string()),
    };
    let lang_meta = metadata_items.iter().find(|m| m.id == lang_id).ok_or_else(|| format!("Language '{}' not found", lang_id))?;
    let font_meta = lang_meta.font.as_ref();
    let game_version = crate::version::get_exe_version(blitz_path).unwrap_or_default();
    let inst_info_path = data_dir.join(INST_INFO_FILE);
    let old_inst: Option<InstInfo> = std::fs::read_to_string(&inst_info_path).ok().and_then(|c| serde_json::from_str(&c).ok());

    // L10n
    let l10n_extract_dir = temp_extract_dir().join("l10n");
    let _ = std::fs::remove_dir_all(&l10n_extract_dir);
    let l10n_etag = download_and_extract_7z(app, blitz_path, &lang_meta.l10n.path, &l10n_extract_dir, proxy_url, "l10n", old_inst.as_ref().map(|i| i.l10n_etag.as_str())).await?;

    let l10n_version: String;
    let mut all_backups: HashMap<String, String> = HashMap::new();
    let mut idx: u32 = 0;

    if l10n_extract_dir.join("metadata.json").exists() {
        let l10n_manifest: ExtractManifest = {
            let content = std::fs::read_to_string(l10n_extract_dir.join("metadata.json")).map_err(|e| format!("Failed to read l10n metadata.json: {}", e))?;
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse l10n metadata.json: {}", e))?
        };
        if !is_version_compatible(&l10n_manifest.supported_game_version, &game_version) {
            let _ = std::fs::remove_dir_all(&l10n_extract_dir);
            return Err(format!("No supported version: artifact requires game version {}, but you have {}", l10n_manifest.supported_game_version, game_version));
        }
        for (src_rel, dst_list) in &l10n_manifest.files {
            let src_file = l10n_extract_dir.join(src_rel);
            if !src_file.is_file() { continue; }
            for dst_rel in dst_list {
                let backup_name = backup_single_file(&data_dir, dst_rel, &fingerprint)?;
                all_backups.insert(dst_rel.clone(), backup_name);
                let dst_path = data_dir.join(dst_rel);
                if let Some(parent) = dst_path.parent() { std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?; }
                std::fs::copy(&src_file, &dst_path).map_err(|e| format!("Failed to install {}: {}", dst_rel, e))?;
            }
            idx += 1;
            if let Some(app) = app { emit_progress(app, blitz_path, "install_l10n", (idx as f64 / l10n_manifest.files.len().max(1) as f64 * 100.0).min(99.0) as u32, "installing_l10n", 0, 0); }
        }
        l10n_version = l10n_manifest.version;
    } else {
        l10n_version = old_inst.as_ref().map(|i| i.loc_version.clone()).unwrap_or_default();
    }
    let _ = std::fs::remove_dir_all(&l10n_extract_dir);

    // Font
    let (font_version, font_etag) = if let Some(fm) = font_meta {
        let font_extract_dir = temp_extract_dir().join("font");
        let _ = std::fs::remove_dir_all(&font_extract_dir);
        let etag = download_and_extract_7z(app, blitz_path, &fm.path, &font_extract_dir, proxy_url, "font", old_inst.as_ref().map(|i| i.font_etag.as_str())).await?;

        if font_extract_dir.join("metadata.json").exists() {
            let font_manifest: ExtractManifest = {
                let content = std::fs::read_to_string(font_extract_dir.join("metadata.json")).map_err(|e| format!("Failed to read font metadata.json: {}", e))?;
                serde_json::from_str(&content).map_err(|e| format!("Failed to parse font metadata.json: {}", e))?
            };
            if !is_version_compatible(&font_manifest.supported_game_version, &game_version) {
                log::warn!("Font not compatible with game {}; skipping.", game_version);
                let _ = std::fs::remove_dir_all(&font_extract_dir);
                (String::new(), String::new())
            } else {
                idx = 0;
                for (src_rel, dst_list) in &font_manifest.files {
                    let src_file = font_extract_dir.join(src_rel);
                    if !src_file.is_file() { continue; }
                    for dst_rel in dst_list {
                        let backup_name = backup_single_file(&data_dir, dst_rel, &fingerprint)?;
                        all_backups.insert(dst_rel.clone(), backup_name);
                        let dst_path = data_dir.join(dst_rel);
                        if let Some(parent) = dst_path.parent() { std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?; }
                        std::fs::copy(&src_file, &dst_path).map_err(|e| format!("Failed to install font {}: {}", dst_rel, e))?;
                    }
                    idx += 1;
                    if let Some(app) = app { emit_progress(app, blitz_path, "install_font", (idx as f64 / font_manifest.files.len().max(1) as f64 * 100.0).min(99.0) as u32, "installing_font", 0, 0); }
                }
                let _ = std::fs::remove_dir_all(&font_extract_dir);
                (font_manifest.version, etag)
            }
        } else {
            (old_inst.as_ref().map(|i| i.font_version.clone()).unwrap_or_default(), etag)
        }
    } else { (String::new(), String::new()) };

    let inst_info = InstInfo { language: lang_id.to_string(), date: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(), loc_version: l10n_version, l10n_etag, font_version, font_etag, font_id: font_meta.map(|f| f.id.clone()).unwrap_or_default(), installed_game_version: game_version, app_version: env!("CARGO_PKG_VERSION").to_string(), backups: all_backups };
    let content = serde_json::to_string_pretty(&inst_info).map_err(|e| format!("Failed to serialize inst info: {}", e))?;
    std::fs::write(&inst_info_path, content).map_err(|e| format!("Failed to write inst info: {}", e))?;
    let _ = std::fs::remove_dir_all(&temp_extract_dir());
    if let Some(app) = app { emit_progress(app, blitz_path, "done", 100, "done", 0, 0); }
    Ok(lang_id.to_string())
}

pub fn full_uninstall(target_dir: &Path) -> Result<String, String> {
    let data_dir = target_dir.join("Data");
    let inst_info_path = data_dir.join(INST_INFO_FILE);
    if !inst_info_path.exists() { return Err("No localization installation found.".to_string()); }
    let content = std::fs::read_to_string(&inst_info_path).map_err(|e| format!("Failed to read inst info: {}", e))?;
    let inst_info: InstInfo = serde_json::from_str(&content).map_err(|e| format!("Failed to parse inst info: {}", e))?;
    let blitz_path = target_dir.to_string_lossy().replace('\\', "/");
    let fingerprint = instance_fingerprint(&blitz_path);
    for (rel_path, backup_name) in &inst_info.backups { restore_single_file(&data_dir, rel_path, backup_name, &fingerprint)?; }
    let _ = std::fs::remove_file(&inst_info_path);
    let bkp_dir = backup_dir(&fingerprint);
    let _ = std::fs::remove_dir_all(&bkp_dir);
    log::info!("Uninstall complete for {}", blitz_path);
    Ok("Uninstall complete".to_string())
}

pub fn check_blitz_status(blitz_path: &str, exe_version: &str) -> BlitzStatus {
    let data_dir = PathBuf::from(blitz_path).join("Data");
    let inst_info_path = data_dir.join(INST_INFO_FILE);
    if let Ok(content) = std::fs::read_to_string(&inst_info_path) {
        if let Ok(info) = serde_json::from_str::<InstInfo>(&content) {
            let is_compatible = is_version_compatible(&info.installed_game_version, exe_version);
            return BlitzStatus { path: blitz_path.to_string(), version: exe_version.to_string(), loc_installed: true, loc_version: info.loc_version, font_version: info.font_version, loc_language: info.language, is_compatible };
        }
    }
    BlitzStatus { path: blitz_path.to_string(), version: exe_version.to_string(), loc_installed: false, loc_version: String::new(), font_version: String::new(), loc_language: String::new(), is_compatible: false }
}

pub fn get_cache_size_bytes() -> u64 {
    let dir = cache_dir();
    if !dir.exists() { return 0; }
    let mut total: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() { if meta.is_file() { total += meta.len(); } }
        }
    }
    total
}

pub fn clear_all_caches() -> Result<(), String> {
    let dir = cache_dir();
    if dir.exists() { std::fs::remove_dir_all(&dir).map_err(|e| format!("Failed to clear cache: {}", e))?; }
    Ok(())
}
