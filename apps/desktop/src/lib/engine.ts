import { invoke } from '@tauri-apps/api/core';
import type { EngineManager, EngineVersion } from '@glyph/ui/application';

/**
 * Desktop engine manager — lists, downloads, and activates Tectonic versions
 * from GitHub releases (Rust `engine` module). Lets users update the LaTeX
 * engine without rebuilding the app.
 */
export const engineManager: EngineManager = {
	label: 'Tectonic',
	list: () => invoke<EngineVersion[]>('list_tectonic_versions'),
	download: (version: string) => invoke<string>('download_tectonic', { version }),
	setActive: (version: string) => invoke<void>('set_active_engine', { version })
};
