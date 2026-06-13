import { settings } from '@glyphx/ui/settings';

/**
 * Feed the native OS theme from Tauri into the settings store, overriding the
 * WebView's `prefers-color-scheme` guess with the real system setting. No-op
 * outside Tauri (e.g. running the SvelteKit dev server in a plain browser).
 * Returns a cleanup function that detaches the listener.
 */
export async function initTauriTheme(): Promise<() => void> {
	if (typeof window === 'undefined') return () => {};

	const isTauri = '__TAURI_INTERNALS__' in window || 'isTauri' in window;
	if (!isTauri) return () => {};

	const { getCurrentWindow } = await import('@tauri-apps/api/window');
	const win = getCurrentWindow();

	const current = await win.theme();
	if (current === 'light' || current === 'dark') settings.setSystemTheme(current);

	return win.onThemeChanged(({ payload }) => {
		if (payload === 'light' || payload === 'dark') settings.setSystemTheme(payload);
	});
}
