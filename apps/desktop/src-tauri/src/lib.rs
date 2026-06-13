mod compile;
mod engine;
mod project;
mod shell_integration;

use std::sync::Mutex;

use tauri::{Emitter, Manager};

/// The file / folder path Glyph was launched to open (via a file association or
/// "Open with Glyph"), captured at startup and handed to the frontend once.
#[derive(Default)]
struct LaunchPath(Mutex<Option<String>>);

/// Pick the first argument that looks like a real path to open (skips flags and
/// the executable itself). File associations pass the target as a plain arg.
fn first_path_arg(args: &[String]) -> Option<String> {
    args.iter()
        .skip(1)
        .find(|a| !a.starts_with('-') && std::path::Path::new(a).exists())
        .cloned()
}

/// Return (and clear) the path Glyph was launched with, if any. The frontend
/// calls this once on startup to open an associated `.tex` / `.glyx` / folder.
#[tauri::command]
fn take_launch_path(state: tauri::State<'_, LaunchPath>) -> Option<String> {
    state.0.lock().ok().and_then(|mut p| p.take())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Single-instance: a second launch (e.g. double-clicking another file)
        // forwards its path to the running window instead of opening a new app.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            if let Some(path) = first_path_arg(&argv) {
                let _ = app.emit("glyph://open-path", path);
            }
            if let Some(win) = app.get_webview_window("main") {
                let _ = win.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(LaunchPath::default())
        .setup(|app| {
            // Capture the path this (first) instance was launched with.
            let args: Vec<String> = std::env::args().collect();
            if let Some(path) = first_path_arg(&args) {
                if let Some(state) = app.try_state::<LaunchPath>() {
                    if let Ok(mut p) = state.0.lock() {
                        *p = Some(path);
                    }
                }
            }
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            compile::compile_latex,
            compile::compile_project,
            compile::prefetch_packages,
            engine::list_tectonic_versions,
            engine::download_tectonic,
            engine::set_active_engine,
            engine::remove_tectonic,
            engine::tectonic_cache_info,
            engine::clear_tectonic_cache,
            project::read_project_files,
            project::read_file_text,
            project::write_file_text,
            project::create_path,
            project::rename_path,
            project::delete_path,
            project::path_exists,
            project::import_zip,
            project::export_zip,
            shell_integration::register_shell_integration,
            take_launch_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
