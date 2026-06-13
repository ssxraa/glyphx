<script lang="ts">
	import { Workbench } from '@glyphx/ui/application';
	import { compileLatex, warmEngine, texliveStatus } from '$lib/compile';
	import { onMount } from 'svelte';

	// Which TeX package server the engine settled on, shown in the status bar.
	let serverNote = $state<string | undefined>(undefined);

	onMount(() => {
		// Start downloading + booting the WASM engine while the user gets oriented,
		// so the first compile doesn't pay the full cold-start cost.
		warmEngine();
		texliveStatus()
			.then((s) => (serverNote = `Packages: ${s.self ? 'your server' : 'public'}`))
			.catch(() => {});
	});
</script>

<svelte:head>
	<title>GlyphX — Editor</title>
</svelte:head>

<Workbench platform="web" compile={compileLatex} statusNote={serverNote} />
