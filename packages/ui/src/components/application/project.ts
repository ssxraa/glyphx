/**
 * ProjectHost — the host-provided bridge to a real, folder-based LaTeX project
 * on disk. The desktop app backs this with Tauri (native dialogs + Rust fs / zip
 * commands); the web build leaves it undefined and the workbench stays on its
 * in-memory demo document.
 *
 * A project is just a folder. The "main file" is the compile target; other
 * files (`\input`, `\includegraphics`, `\bibliography`, …) resolve relative to
 * it on disk, exactly like Overleaf / a normal local LaTeX project.
 */

/** A file inside a project: stable absolute path + forward-slash relative path. */
export type ProjectFile = { abs: string; rel: string };

export type ProjectHost = {
	/** Native folder picker → chosen absolute path, or null if cancelled. */
	pickFolder: (title?: string) => Promise<string | null>;
	/** Native file picker for import (.zip / .glyx / .tex) → path, or null. */
	pickImportFile: () => Promise<string | null>;

	/** Recursively list a project folder's files (junk / VCS dirs excluded). */
	readFiles: (root: string) => Promise<ProjectFile[]>;
	/** Read a text file (rejects binary / non-UTF-8). */
	readFile: (abs: string) => Promise<string>;
	/** Write (overwrite) a text file, creating parent folders. */
	writeFile: (abs: string, content: string) => Promise<void>;
	/** Create a new empty file (or folder when `dir`). */
	createEntry: (abs: string, dir: boolean) => Promise<void>;
	/** Rename / move a file or folder. */
	rename: (from: string, to: string) => Promise<void>;
	/** Delete a file or folder (folders recursively). */
	remove: (abs: string) => Promise<void>;
	/** Whether a path exists. */
	exists: (abs: string) => Promise<boolean>;

	/** Import a `.zip`: prompts for a destination, extracts, returns the new root (or null). */
	importZip: (zipPath: string) => Promise<string | null>;
	/** Export a project folder to a `.zip` via a native save dialog. False = cancelled. */
	exportZip: (root: string, defaultName: string) => Promise<boolean>;

	/** Path GlyphX was launched to open (file association); consumed once. */
	takeLaunchPath?: () => Promise<string | null>;
	/** Subscribe to later open-path events (a second launch). Returns an unsubscribe fn. */
	onOpenPath?: (cb: (path: string) => void) => Promise<() => void>;
	/** Best-effort: register the "Open with GlyphX" folder context menu (Windows). */
	registerShellIntegration?: () => Promise<string>;
};
