<script lang="ts">
	import { Workbench } from '@glyph/ui/application';
	import { compileLatex } from '$lib/compile';
	import { engineManager } from '$lib/engine';
</script>

<svelte:head>
	<title>Glyph</title>
</svelte:head>

<Workbench platform="desktop" compile={compileLatex} engine={engineManager} />
