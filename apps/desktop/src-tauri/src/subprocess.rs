//! Shared spawn helper for the Rust commands.
//!
//! On Windows, a child process spawned from a GUI (windowed) app gets its own
//! console allocated — visible as a black window that flashes on screen every
//! time we run Tectonic / latexmk / git / reg. It's especially noticeable in
//! release builds (debug builds already own a console). Passing the
//! `CREATE_NO_WINDOW` creation flag suppresses that console so subprocesses run
//! invisibly. The flag — and the whole trait — is a no-op on other platforms.

use std::process::Command;

/// `CREATE_NO_WINDOW` — run the child without allocating a console window.
/// <https://learn.microsoft.com/windows/win32/procthread/process-creation-flags>
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Adds `.no_window()` to [`Command`], suppressing the child's console window on
/// Windows (no-op elsewhere). Call it anywhere before `output()` / `status()` /
/// `spawn()`; it returns `&mut Command` so it chains like the builder methods.
pub trait CommandExt {
    fn no_window(&mut self) -> &mut Self;
}

impl CommandExt for Command {
    fn no_window(&mut self) -> &mut Self {
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt as _;
            self.creation_flags(CREATE_NO_WINDOW);
        }
        self
    }
}
