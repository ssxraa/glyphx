//! OS shell integration.
//!
//! `.tex` and `.glyx` *file* associations are declared in `tauri.conf.json`
//! (`bundle.fileAssociations`) and registered by the installer. Folder
//! association ("Open with GlyphX" on a directory) is non-standard and not
//! covered by Tauri, so we register it ourselves on Windows via a per-user
//! registry entry (HKCU — no admin prompt). Best-effort: a failure is reported
//! but never fatal.

/// Register a "Open with GlyphX" entry on the folder right-click menu (Windows).
/// No-op (with an explanatory message) on other platforms.
#[tauri::command]
pub fn register_shell_integration() -> Result<String, String> {
    #[cfg(windows)]
    {
        register_windows_folder_menu()
    }
    #[cfg(not(windows))]
    {
        Ok("Folder association is only set up on Windows; .tex / .glyx files are \
            associated via the installer on this platform."
            .to_string())
    }
}

#[cfg(windows)]
fn reg_escape(s: &str) -> String {
    // In .reg string values, backslashes are doubled and quotes are backslash-escaped.
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(windows)]
fn register_windows_folder_menu() -> Result<String, String> {
    use std::process::Command;

    let exe = std::env::current_exe()
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .into_owned();
    let exe_esc = reg_escape(&exe);
    // `%V` resolves to the clicked folder (or the background folder).
    let command_value = format!("\\\"{exe_esc}\\\" \\\"%V\\\"");

    let reg = format!(
        "Windows Registry Editor Version 5.00\r\n\r\n\
         [HKEY_CURRENT_USER\\Software\\Classes\\Directory\\shell\\GlyphXOpen]\r\n\
         @=\"Open with GlyphX\"\r\n\
         \"Icon\"=\"{exe_esc}\"\r\n\r\n\
         [HKEY_CURRENT_USER\\Software\\Classes\\Directory\\shell\\GlyphXOpen\\command]\r\n\
         @=\"{command_value}\"\r\n\r\n\
         [HKEY_CURRENT_USER\\Software\\Classes\\Directory\\Background\\shell\\GlyphXOpen]\r\n\
         @=\"Open with GlyphX\"\r\n\
         \"Icon\"=\"{exe_esc}\"\r\n\r\n\
         [HKEY_CURRENT_USER\\Software\\Classes\\Directory\\Background\\shell\\GlyphXOpen\\command]\r\n\
         @=\"{command_value}\"\r\n"
    );

    let mut tmp = std::env::temp_dir();
    tmp.push("glyphx-folder-assoc.reg");
    std::fs::write(&tmp, reg).map_err(|e| e.to_string())?;

    let status = Command::new("reg")
        .arg("import")
        .arg(&tmp)
        .status()
        .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&tmp);

    if status.success() {
        Ok("Added \"Open with GlyphX\" to the folder right-click menu.".to_string())
    } else {
        Err(format!("reg import failed (exit {:?})", status.code()))
    }
}
