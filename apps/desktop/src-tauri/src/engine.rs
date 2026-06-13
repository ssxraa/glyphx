//! Managed Tectonic engine versions.
//!
//! Lists and downloads Tectonic builds from GitHub releases into the app-data
//! directory, so users can choose / update the LaTeX engine at runtime without
//! rebuilding the app. The active engine is what `compile::find_tectonic`
//! prefers.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::Manager;

const REPO: &str = "tectonic-typesetting/tectonic";
const USER_AGENT: &str = "glyph-editor";

#[derive(Serialize, Clone)]
pub struct EngineVersion {
    pub version: String,
    pub tag: String,
    pub installed: bool,
    pub active: bool,
}

fn bin_ext() -> &'static str {
    if cfg!(windows) {
        ".exe"
    } else {
        ""
    }
}

fn engines_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    let dir = app.path().app_data_dir().ok()?.join("engines");
    std::fs::create_dir_all(&dir).ok()?;
    Some(dir)
}

fn installed_path(dir: &Path, version: &str) -> PathBuf {
    dir.join(format!("tectonic-{version}{}", bin_ext()))
}

fn active_marker(dir: &Path) -> PathBuf {
    dir.join("active.txt")
}

fn read_active(dir: &Path) -> Option<String> {
    std::fs::read_to_string(active_marker(dir))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Path to the active managed engine, if one is installed. Used by the compiler.
pub fn active_engine_path(app: &tauri::AppHandle) -> Option<PathBuf> {
    let dir = engines_dir(app)?;

    if let Some(version) = read_active(&dir) {
        let p = installed_path(&dir, &version);
        if p.exists() {
            return Some(p);
        }
    }

    // Otherwise fall back to any installed engine.
    for entry in std::fs::read_dir(&dir).ok()?.flatten() {
        let path = entry.path();
        let name = path.file_name()?.to_string_lossy().into_owned();
        if !name.starts_with("tectonic-") {
            continue;
        }
        let is_exe = name.ends_with(".exe");
        if cfg!(windows) == is_exe {
            return Some(path);
        }
    }
    None
}

/// Whether a release asset matches the current platform.
fn asset_matches(name: &str) -> bool {
    let target: Option<(&str, &str)> = if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Some(("x86_64-pc-windows-msvc", ".zip"))
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some(("aarch64-apple-darwin", ".tar.gz"))
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some(("x86_64-apple-darwin", ".tar.gz"))
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some(("x86_64-unknown-linux-gnu", ".tar.gz"))
    } else {
        None
    };

    match target {
        Some((triple, ext)) => name.contains(triple) && name.ends_with(ext),
        None => false,
    }
}

#[derive(Deserialize)]
struct GhRelease {
    tag_name: String,
    #[serde(default)]
    assets: Vec<GhAsset>,
}

#[derive(Deserialize, Clone)]
struct GhAsset {
    name: String,
    browser_download_url: String,
}

async fn fetch_releases() -> Result<Vec<GhRelease>, String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases?per_page=40");
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("GitHub API returned {}", resp.status()));
    }
    resp.json::<Vec<GhRelease>>()
        .await
        .map_err(|e| e.to_string())
}

/// List Tectonic versions that have a binary for this platform.
#[tauri::command]
pub async fn list_tectonic_versions(app: tauri::AppHandle) -> Result<Vec<EngineVersion>, String> {
    let dir = engines_dir(&app).ok_or("no app data directory")?;
    let active = read_active(&dir);
    let releases = fetch_releases().await?;

    let mut out = Vec::new();
    for rel in releases {
        // The CLI is released under tags like `tectonic@0.16.9`.
        let Some(version) = rel.tag_name.strip_prefix("tectonic@").map(str::to_string) else {
            continue;
        };
        if !rel.assets.iter().any(|a| asset_matches(&a.name)) {
            continue;
        }
        out.push(EngineVersion {
            installed: installed_path(&dir, &version).exists(),
            active: active.as_deref() == Some(version.as_str()),
            version,
            tag: rel.tag_name,
        });
    }
    Ok(out)
}

/// Download a Tectonic version, extract its binary into app-data, make it active.
#[tauri::command]
pub async fn download_tectonic(app: tauri::AppHandle, version: String) -> Result<String, String> {
    let dir = engines_dir(&app).ok_or("no app data directory")?;
    let releases = fetch_releases().await?;
    let tag = format!("tectonic@{version}");

    let rel = releases
        .into_iter()
        .find(|r| r.tag_name == tag)
        .ok_or("version not found")?;
    let asset = rel
        .assets
        .into_iter()
        .find(|a| asset_matches(&a.name))
        .ok_or("no Tectonic build for this platform")?;

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;
    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    let target = installed_path(&dir, &version);
    extract_binary(&bytes, &asset.name, &target)?;
    std::fs::write(active_marker(&dir), &version).map_err(|e| e.to_string())?;

    Ok(target.to_string_lossy().into_owned())
}

/// Switch the active engine to an already-installed version.
#[tauri::command]
pub async fn set_active_engine(app: tauri::AppHandle, version: String) -> Result<(), String> {
    let dir = engines_dir(&app).ok_or("no app data directory")?;
    if !installed_path(&dir, &version).exists() {
        return Err("that version is not installed".into());
    }
    std::fs::write(active_marker(&dir), &version).map_err(|e| e.to_string())
}

fn extract_binary(bytes: &[u8], asset_name: &str, target: &Path) -> Result<(), String> {
    if asset_name.ends_with(".zip") {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| e.to_string())?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().replace('\\', "/");
            if name.ends_with("tectonic.exe") || name.ends_with("tectonic") {
                let mut buf = Vec::new();
                std::io::copy(&mut entry, &mut buf).map_err(|e| e.to_string())?;
                std::fs::write(target, &buf).map_err(|e| e.to_string())?;
                return set_executable(target);
            }
        }
        Err("no Tectonic binary inside the archive".into())
    } else if asset_name.ends_with(".tar.gz") {
        let gz = flate2::read::GzDecoder::new(Cursor::new(bytes));
        let mut archive = tar::Archive::new(gz);
        for entry in archive.entries().map_err(|e| e.to_string())? {
            let mut entry = entry.map_err(|e| e.to_string())?;
            let path = entry.path().map_err(|e| e.to_string())?.into_owned();
            if path.file_name().map(|n| n == "tectonic").unwrap_or(false) {
                let mut buf = Vec::new();
                std::io::copy(&mut entry, &mut buf).map_err(|e| e.to_string())?;
                std::fs::write(target, &buf).map_err(|e| e.to_string())?;
                return set_executable(target);
            }
        }
        Err("no Tectonic binary inside the archive".into())
    } else {
        Err("unknown archive format".into())
    }
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)
        .map_err(|e| e.to_string())?
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).map_err(|e| e.to_string())
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<(), String> {
    Ok(())
}
