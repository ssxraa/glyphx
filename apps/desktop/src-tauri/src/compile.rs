//! LaTeX compilation via the Tectonic engine.
//!
//! Tectonic ships heavy native dependencies, so rather than linking the crate
//! (which needs vcpkg/harfbuzz/freetype/… on Windows) we drive the Tectonic
//! binary as a subprocess. The binary is resolved, in order, from:
//!   1. the `GLYPH_TECTONIC_BIN` environment variable,
//!   2. a bundled sidecar next to the app executable, then
//!   3. `tectonic` on `PATH`.
//!
//! This keeps `cargo build` free of native-lib requirements while still giving
//! a real, offline LaTeX → PDF pipeline once the binary is available. (Tectonic
//! fetches its TeX bundle from the web on first run, then works fully offline.)

use std::path::PathBuf;
use std::process::Command;

use base64::{engine::general_purpose, Engine as _};
use serde::Serialize;

#[derive(Serialize)]
pub struct CompileResult {
    /// Whether a PDF was produced.
    pub success: bool,
    /// The compiled PDF, base64-encoded (None on failure).
    pub pdf_base64: Option<String>,
    /// Tectonic's log / stderr output (warnings, errors).
    pub log: String,
    /// Human-readable message when something went wrong.
    pub message: Option<String>,
    /// Decompressed SyncTeX data (`.synctex.gz` contents) for reverse search.
    pub synctex: Option<String>,
}

impl CompileResult {
    fn failure(message: impl Into<String>, log: impl Into<String>) -> Self {
        Self {
            success: false,
            pdf_base64: None,
            log: log.into(),
            message: Some(message.into()),
            synctex: None,
        }
    }
}

/// Ensure a fontconfig config exists and return its path (Windows only).
///
/// Tectonic's XeTeX engine initializes fontconfig for font lookups; on Windows
/// there is no default config, which prints "Cannot load default config file"
/// and can destabilize font handling. We write a minimal config pointing at the
/// system fonts dir with a writable cache so initialization always succeeds.
#[cfg(windows)]
fn ensure_fontconfig(app: &tauri::AppHandle) -> Option<PathBuf> {
    use tauri::Manager;
    let base = app.path().app_data_dir().ok()?;
    std::fs::create_dir_all(&base).ok()?;
    let cache = base.join("fontconfig-cache");
    std::fs::create_dir_all(&cache).ok()?;
    let conf = base.join("fonts.conf");
    if !conf.exists() {
        let xml = format!(
            "<?xml version=\"1.0\"?>\n<!DOCTYPE fontconfig SYSTEM \"fonts.dtd\">\n<fontconfig>\n  <dir>C:/Windows/Fonts</dir>\n  <cachedir>{}</cachedir>\n</fontconfig>\n",
            cache.to_string_lossy().replace('\\', "/")
        );
        std::fs::write(&conf, xml).ok()?;
    }
    Some(conf)
}

#[cfg(not(windows))]
fn ensure_fontconfig(_app: &tauri::AppHandle) -> Option<PathBuf> {
    None
}

/// Pre-download a set of common LaTeX packages into the cache so the first
/// offline compile works. Best-effort: returns the compile result of a small
/// document that pulls the packages in.
#[tauri::command]
pub async fn prefetch_packages(app: tauri::AppHandle) -> Result<CompileResult, String> {
    const WARM: &str = r"\documentclass{article}
\usepackage{amsmath,amssymb,graphicx,geometry,booktabs,enumitem,xcolor,titlesec,fancyhdr,tabularx,microtype,parskip,multicol,listings}
\usepackage[hidelinks]{hyperref}
\usepackage[english]{babel}
\begin{document}Warming the package cache.\end{document}";
    compile_latex(app, WARM.to_string(), Some(false)).await
}

/// Read and gunzip `<stem>.synctex.gz` from the output dir, if present.
fn read_synctex(dir: &std::path::Path, stem: &str) -> Option<String> {
    let path = dir.join(format!("{stem}.synctex.gz"));
    let bytes = std::fs::read(path).ok()?;
    let mut decoder = flate2::read::GzDecoder::new(&bytes[..]);
    let mut text = String::new();
    use std::io::Read as _;
    decoder.read_to_string(&mut text).ok()?;
    Some(text)
}

const BIN_NAMES: [&str; 2] = ["tectonic.exe", "tectonic"];

/// Locate the Tectonic executable, in priority order:
///   1. `GLYPH_TECTONIC_BIN`
///   2. an engine downloaded into the app-data dir (managed versions)
///   3. next to the app executable (bundled sidecar) or a `binaries/` dir in
///      any ancestor (covers `tauri dev`, where the exe is under target/debug)
///   4. `tectonic` on `PATH`
pub fn find_tectonic(app: &tauri::AppHandle) -> PathBuf {
    if let Ok(custom) = std::env::var("GLYPH_TECTONIC_BIN") {
        let pb = PathBuf::from(custom);
        if pb.exists() {
            return pb;
        }
    }

    if let Some(managed) = crate::engine::active_engine_path(app) {
        return managed;
    }

    if let Ok(exe) = std::env::current_exe() {
        let mut dir = exe.parent();
        for _ in 0..7 {
            let Some(d) = dir else { break };
            for name in BIN_NAMES {
                let direct = d.join(name);
                if direct.exists() {
                    return direct;
                }
                let in_binaries = d.join("binaries").join(name);
                if in_binaries.exists() {
                    return in_binaries;
                }
            }
            dir = d.parent();
        }
    }

    // Fall back to PATH (e.g. installed via choco/scoop/apt).
    PathBuf::from("tectonic")
}

/// Run Tectonic on `main_tex`, writing build artifacts to `outdir`, and assemble
/// the result. Tectonic resolves `\input`, `\includegraphics`, `\bibliography`
/// etc. relative to the main file's own directory, so multi-file projects work
/// as long as `main_tex` lives in the project folder. The PDF / log / synctex
/// are named after the main file's stem (e.g. `report.tex` → `report.pdf`).
fn run_tectonic(
    app: &tauri::AppHandle,
    main_tex: &std::path::Path,
    outdir: &std::path::Path,
    shell_escape: bool,
) -> CompileResult {
    let stem = main_tex
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "main".to_string());

    let bin = find_tectonic(app);
    let mut cmd = Command::new(&bin);
    cmd.arg("--outdir")
        .arg(outdir)
        .arg("--keep-logs")
        .arg("--synctex")
        .arg("--chatter")
        .arg("minimal")
        // Keep compiling through recoverable TeX errors (undefined refs / macros,
        // bad boxes, …) and still emit a best-effort PDF — the Overleaf / latexmk
        // `nonstopmode` behaviour. Errors are surfaced in the log / Problems panel.
        .arg("-Z")
        .arg("continue-on-errors");
    // Shell escape (`\write18`) for packages like minted / gnuplot — opt-in
    // (it lets a document run system commands). Set the working dir to the main
    // file's folder so relative paths (e.g. for Pygments) resolve.
    if shell_escape {
        let cwd = main_tex.parent().unwrap_or(outdir);
        cmd.arg("-Z")
            .arg(format!("shell-escape-cwd={}", cwd.to_string_lossy()));
    }
    cmd.arg(main_tex);
    // Deterministic, app-managed package cache (so Settings can show/clear/warm it).
    if let Some(cache) = crate::engine::cache_dir(app) {
        cmd.env("TECTONIC_CACHE_DIR", cache);
    }
    // Give fontconfig a real config so XeTeX font lookups don't emit the
    // "Cannot load default config file" noise (Windows; harmless elsewhere).
    if let Some(conf) = ensure_fontconfig(app) {
        cmd.env("FONTCONFIG_FILE", conf);
    }

    let output = match cmd.output() {
        Ok(o) => o,
        Err(e) => {
            return CompileResult::failure(
                format!(
                    "Could not run Tectonic ({}). Install it (e.g. `choco install tectonic`) \
                     or set GLYPH_TECTONIC_BIN. Underlying error: {e}",
                    bin.display()
                ),
                String::new(),
            );
        }
    };

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    // The TeX engine log (`<stem>.log`) carries the structured `! error` blocks,
    // `l.<n>` line numbers and `LaTeX Warning ... on input line <n>` that the
    // frontend parses. Combine it with Tectonic's concise stderr summary.
    let tex_log = std::fs::read_to_string(outdir.join(format!("{stem}.log"))).unwrap_or_default();
    let log = if tex_log.is_empty() {
        stderr.clone()
    } else {
        format!("{}\n{}", stderr.trim_end(), tex_log)
    };

    // Prefer emitting whatever PDF was produced. With `continue-on-errors`,
    // Tectonic renders a best-effort PDF for recoverable errors (and still exits
    // non-zero), so a present PDF means "show it" — the errors live in the log
    // and surface in the Problems panel, exactly like Overleaf.
    let pdf_path = outdir.join(format!("{stem}.pdf"));
    if let Ok(bytes) = std::fs::read(&pdf_path) {
        if !output.status.success() {
            eprintln!(
                "[glyph] compiled with errors (exit {:?}) — showing best-effort PDF",
                output.status.code()
            );
        }
        return CompileResult {
            success: true,
            pdf_base64: Some(general_purpose::STANDARD.encode(bytes)),
            log,
            message: None,
            synctex: read_synctex(outdir, &stem),
        };
    }

    // No PDF at all — a genuine failure. Mirror it to the dev terminal too.
    eprintln!(
        "[glyph] LaTeX compilation failed (exit {:?}):\n{}",
        output.status.code(),
        if stderr.trim().is_empty() {
            tex_log.as_str()
        } else {
            stderr.trim()
        }
    );
    // No TeX log written ⇒ the engine itself crashed (rather than a normal
    // LaTeX error). Most often an OpenType icon font (e.g. fontawesome5) on
    // the stable engine — the nightly build fixes it.
    let message = if tex_log.trim().is_empty() {
        format!(
            "The LaTeX engine exited unexpectedly (code {:?}) without producing a log — \
             a package likely crashed it (often an icon font such as fontawesome5 on the \
             stable engine). Try the Nightly engine in Settings → Engine.",
            output.status.code()
        )
    } else {
        "LaTeX compilation failed — no PDF was produced. See the Problems panel.".to_string()
    };
    CompileResult::failure(message, log)
}

/// Compile a standalone LaTeX `source` string into a PDF (no project on disk).
/// Used for the in-memory scratch / sample document and the web fallback.
#[tauri::command]
pub async fn compile_latex(
    app: tauri::AppHandle,
    source: String,
    shell_escape: Option<bool>,
) -> Result<CompileResult, String> {
    let dir = tempfile::tempdir().map_err(|e| e.to_string())?;
    let tex_path = dir.path().join("main.tex");
    std::fs::write(&tex_path, &source).map_err(|e| e.to_string())?;
    Ok(run_tectonic(
        &app,
        &tex_path,
        dir.path(),
        shell_escape.unwrap_or(false),
    ))
}

/// Compile a multi-file project on disk. `root` is the project folder and `main`
/// is the main `.tex` file's path (relative to `root`, or absolute). Build
/// artifacts go to a throwaway temp dir so the project folder stays clean, while
/// Tectonic still resolves includes relative to the main file inside `root`.
#[tauri::command]
pub async fn compile_project(
    app: tauri::AppHandle,
    root: String,
    main: String,
    shell_escape: Option<bool>,
) -> Result<CompileResult, String> {
    let root = PathBuf::from(&root);
    let main_path = {
        let p = PathBuf::from(&main);
        if p.is_absolute() {
            p
        } else {
            root.join(&main)
        }
    };
    if !main_path.exists() {
        return Ok(CompileResult::failure(
            format!("Main file not found: {}", main_path.display()),
            String::new(),
        ));
    }
    let out = tempfile::tempdir().map_err(|e| e.to_string())?;
    Ok(run_tectonic(
        &app,
        &main_path,
        out.path(),
        shell_escape.unwrap_or(false),
    ))
}
