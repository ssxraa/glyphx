<script lang="ts">
	import { Workbench } from '@glyph/ui/application';
	import { compileLatex } from '$lib/compile';
</script>

<svelte:head>
	<title>Glyph — Editor</title>
</svelte:head>

<Workbench platform="web" compile={compileLatex} />
