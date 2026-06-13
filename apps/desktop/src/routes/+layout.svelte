<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { settings } from '@glyph/ui/settings';
	import { initTauriTheme } from '$lib/tauri-theme';

	let { children } = $props();

	// Keep `.dark` on <html> in sync with the resolved theme.
	$effect(() => {
		settings.apply();
	});

	// On desktop, let Tauri drive system-theme detection (overrides matchMedia).
	$effect(() => {
		let active = true;
		let cleanup = () => {};
		initTauriTheme().then((fn) => {
			if (active) cleanup = fn;
			else fn();
		});
		return () => {
			active = false;
			cleanup();
		};
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
{@render children()}
