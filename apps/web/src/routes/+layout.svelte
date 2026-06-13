<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { settings } from '@glyph/ui/settings';

	let { children } = $props();

	// Keep `.dark` on <html> in sync with the resolved theme. Re-runs on toggle
	// and on cross-window storage sync (PersistedState updates `resolved`).
	$effect(() => {
		settings.apply();
	});
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>
{@render children()}
