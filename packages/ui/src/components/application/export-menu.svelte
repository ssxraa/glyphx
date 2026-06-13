<script lang="ts">
	import { Button, type ButtonSize } from '@glyph/ui/button';
	import { toast } from '@glyph/ui/sonner';
	import {
		IconDownload,
		IconChevronDown,
		IconFileTypePdf,
		IconFileText,
		IconCode,
		IconCopy,
		IconCheck,
		IconShare2
	} from '@tabler/icons-svelte';

	/**
	 * ExportMenu — Export / Share. PDF uses browser print-to-PDF of the preview
	 * for now (a print stylesheet in the workbench isolates the preview); real
	 * Tectonic-compiled PDF replaces it once the engine is wired. `.tex` and
	 * `.txt` download immediately.
	 */
	let {
		source = '',
		filename = 'main.tex',
		pdfUrl,
		size = 'sm'
	}: { source?: string; filename?: string; pdfUrl?: string; size?: ButtonSize } = $props();

	let open = $state(false);
	let root = $state<HTMLDivElement>();

	const baseName = $derived(filename.replace(/\.[^./\\]+$/, '') || 'document');

	function download(name: string, content: string, type: string) {
		const blob = new Blob([content], { type });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = name;
		a.click();
		URL.revokeObjectURL(url);
		open = false;
	}

	let copied = $state(false);

	function exportPdf() {
		open = false;
		if (pdfUrl) {
			// A real Tectonic-compiled PDF exists — download it directly.
			const a = document.createElement('a');
			a.href = pdfUrl;
			a.download = `${baseName}.pdf`;
			a.click();
			toast.success(`Exported ${baseName}.pdf`);
			return;
		}
		// No compiled PDF yet — fall back to print-to-PDF of the preview.
		toast.message('Opening the print dialog…');
		requestAnimationFrame(() => window.print());
	}
	const exportTex = () => {
		download(`${baseName}.tex`, source, 'application/x-tex');
		toast.success(`Exported ${baseName}.tex`);
	};
	const exportTxt = () => {
		download(`${baseName}.txt`, source, 'text/plain');
		toast.success(`Exported ${baseName}.txt`);
	};

	async function copySource() {
		try {
			await navigator.clipboard.writeText(source);
			copied = true;
			toast.success('Source copied to clipboard');
			setTimeout(() => (copied = false), 1500);
		} catch {
			toast.error('Could not copy — clipboard blocked');
		}
	}

	type Item = { icon: typeof IconDownload; label: string; hint?: string; run: () => void };
	const items = $derived<Item[]>([
		{ icon: IconFileTypePdf, label: 'Export PDF', hint: pdfUrl ? 'Download' : 'Print', run: exportPdf },
		{ icon: IconCode, label: 'Export .tex', run: exportTex },
		{ icon: IconFileText, label: 'Export plain text', run: exportTxt },
		{
			icon: copied ? IconCheck : IconCopy,
			label: copied ? 'Copied' : 'Copy source',
			run: copySource
		}
	]);

	$effect(() => {
		if (!open) return;
		const onDown = (e: MouseEvent) => {
			if (root && !root.contains(e.target as Node)) open = false;
		};
		const onKey = (e: KeyboardEvent) => {
			if (e.key === 'Escape') open = false;
		};
		document.addEventListener('mousedown', onDown);
		document.addEventListener('keydown', onKey);
		return () => {
			document.removeEventListener('mousedown', onDown);
			document.removeEventListener('keydown', onKey);
		};
	});
</script>

<div class="relative" bind:this={root}>
	<Button variant="outline" {size} onclick={() => (open = !open)} aria-expanded={open}>
		<IconDownload size={14} />
		Export
		<IconChevronDown size={13} />
	</Button>

	{#if open}
		<div
			class="bg-popover text-popover-foreground border-border shadow-craft-floating absolute right-0 z-50 mt-1 w-52 rounded-md border p-1"
			role="menu"
		>
			{#each items as item (item.label)}
				{@const Icon = item.icon}
				<button
					class="hover:bg-muted flex w-full items-center gap-2.5 rounded px-2 py-1.5 text-left text-sm transition-colors"
					role="menuitem"
					onclick={item.run}
				>
					<Icon size={16} class="text-muted-foreground" />
					<span class="flex-1">{item.label}</span>
					{#if item.hint}
						<span class="text-muted-foreground/60 text-xs">{item.hint}</span>
					{/if}
				</button>
			{/each}

			<div class="bg-border my-1 h-px"></div>

			<button
				class="text-muted-foreground flex w-full cursor-not-allowed items-center gap-2.5 rounded px-2 py-1.5 text-left text-sm"
				role="menuitem"
				disabled
			>
				<IconShare2 size={16} />
				<span class="flex-1">Share link</span>
				<span class="text-muted-foreground/60 text-xs">Soon</span>
			</button>
		</div>
	{/if}
</div>
