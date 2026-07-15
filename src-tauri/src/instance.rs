use std::collections::HashSet;
use std::path::{Path, PathBuf};
use winreg::enums::*;
use winreg::RegKey;

/// Drive types to skip when enumerating drives.
/// 1 = DRIVE_NO_ROOT_DIR, 5 = DRIVE_CDROM, 6 = DRIVE_RAMDISK
const DRIVE_TYPES_SKIP: [u32; 3] = [1, 5, 6];

/// Game ID for Tanks Blitz Live server.
const TANKS_BLITZ_GAME_ID: &str = "WOTB.RU.PRODUCTION";

/// Main executable name.
const GAME_EXE: &str = "tanksblitz.exe";

/// Common fallback path suffix for scanning.
const COMMON_SUFFIX: &str = "Games/Tanks_Blitz";

/// Collect available drive letters (A-Z) that exist and are not CD-ROM / RAM disk / no-root.
fn find_all_drives() -> Vec<String> {
    let mut drives = Vec::new();
    for d in 'A'..='Z' {
        let root = format!("{}:\\", d);
        if !Path::new(&root).exists() {
            continue;
        }
        let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
        let drive_type = unsafe { windows_sys::GetDriveTypeW(wide.as_ptr()) };
        if DRIVE_TYPES_SKIP.contains(&drive_type) {
            continue;
        }
        drives.push(format!("{}:/", d));
    }
    drives
}

/// Call Windows GetDriveTypeW via windows-sys FFI.
mod windows_sys {
    #[link(name = "kernel32")]
    extern "system" {
        pub fn GetDriveTypeW(lpRootPathName: *const u16) -> u32;
    }
}

// ── Registry-based detection ──

/// Locate LGC install directory from the Windows registry.
/// Returns the directory containing `preferences.xml`, or None.
fn find_lgc_dir_from_registry() -> Option<PathBuf> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey(r"Software\Classes\lgc\DefaultIcon")
        .ok()?;
    let raw: String = key.get_value("").unwrap_or_default();

    if raw.is_empty() {
        return None;
    }

    // Value is typically `"C:\...\lgc.exe",0` — extract the path part.
    let path_str = raw.trim_matches('"');
    let path_str = if let Some(comma_pos) = path_str.rfind(',') {
        let candidate = &path_str[..comma_pos];
        let lower = candidate.to_lowercase();
        if lower.ends_with(".exe") || lower.ends_with(".dll") || lower.ends_with(".ico") {
            candidate
        } else {
            path_str
        }
    } else {
        path_str
    };

    let exe_path = PathBuf::from(path_str);
    let parent = exe_path.parent()?;
    let prefs = parent.join("preferences.xml");
    if prefs.is_file() {
        Some(parent.to_path_buf())
    } else {
        None
    }
}

/// Parse LGC's `preferences.xml` and return the working directories of all
/// games whose `game_id` equals `WOTB.RU.PRODUCTION`.
fn parse_preferences_xml(lgc_dir: &Path) -> Vec<PathBuf> {
    let prefs_path = lgc_dir.join("preferences.xml");
    let xml_str = match std::fs::read_to_string(&prefs_path) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let doc = match roxmltree::Document::parse(&xml_str) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    // Locate the <games> block under <application>/<games_manager>/<games>
    let games_node = match doc
        .descendants()
        .find(|n| n.has_tag_name("games")) {
        Some(n) => n,
        None => return Vec::new(),
    };

    let mut result = Vec::new();

    for game_node in games_node.children().filter(|n| n.has_tag_name("game")) {
        // Check game_id
        let game_id = game_node
            .children()
            .find(|n| n.has_tag_name("game_id"))
            .and_then(|n| n.text())
            .unwrap_or("");

        if game_id != TANKS_BLITZ_GAME_ID {
            continue;
        }

        // Get working_dir
        if let Some(wd) = game_node
            .children()
            .find(|n| n.has_tag_name("working_dir"))
            .and_then(|n| n.text())
        {
            let path = PathBuf::from(wd);
            if validate_instance_path(&path) {
                result.push(path);
            }
        }
    }

    result
}

/// Validate that a directory is a real Tanks Blitz installation.
/// Checks for `tanksblitz.exe` and `Data/` subdirectory.
pub fn validate_instance_path(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    if !path.join(GAME_EXE).is_file() {
        return false;
    }
    if !path.join("Data").is_dir() {
        return false;
    }
    true
}

/// Scan for Tanks Blitz instances via LGC registry → preferences.xml.
fn find_from_registry() -> Vec<(String, String)> {
    let lgc_dir = match find_lgc_dir_from_registry() {
        Some(d) => d,
        None => {
            log::info!("LGC registry key not found. Skipping registry scan.");
            return Vec::new();
        }
    };

    let instances = parse_preferences_xml(&lgc_dir);

    instances
        .into_iter()
        .map(|p| {
            let path_str = p.to_string_lossy().replace('\\', "/");
            log::info!("Tanks Blitz instance found via registry: {}", path_str);
            (path_str, "production".to_string())
        })
        .collect()
}

// ── Common-paths fallback ──

/// Scan common paths on all suitable drives for Tanks Blitz installations.
fn find_from_common_paths() -> Vec<(String, String)> {
    let drives = find_all_drives();
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::new();

    for drive in &drives {
        let candidate = PathBuf::from(drive).join(COMMON_SUFFIX);
        if candidate.is_dir() {
            let path_str = candidate.to_string_lossy().replace('\\', "/");
            if seen.insert(path_str.clone()) && validate_instance_path(&candidate) {
                log::info!("Tanks Blitz instance found via common paths: {}", path_str);
                result.push((path_str, "production".to_string()));
            }
        }
    }

    result
}

// ── Public API ──

/// Find all Tanks Blitz instances.
/// Registry detection first, then common-paths fallback.
/// Returns a deduplicated list of `(path, type_code)` tuples.
pub fn find_all_instances() -> Vec<(String, String)> {
    log::info!("Starting Tanks Blitz instance detection...");

    let registry_results = find_from_registry();
    let common_results = find_from_common_paths();

    let mut seen: HashSet<String> = HashSet::new();
    let mut final_list: Vec<(String, String)> = Vec::new();

    for (path, type_code) in registry_results.into_iter().chain(common_results) {
        if seen.insert(path.clone()) {
            final_list.push((path, type_code));
        }
    }

    log::info!("Detection finished. Found {} instances.", final_list.len());
    final_list
}
