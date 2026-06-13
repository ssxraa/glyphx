<script lang="ts" module>
	// LaTeX-only for now. Typst returns later as an add-on engine.
	export type EditorEngine = 'latex';

	const SAMPLE_LATEX = String.raw`% Glyph — LaTeX document
\documentclass{article}
\usepackage{amsmath}

\title{Hello from Glyph}
\author{}
\date{}

\begin{document}
\maketitle

Glyph compiles \LaTeX{} entirely on your machine with Tectonic.
Nothing is uploaded. Nothing leaves this device.

\begin{equation}
  E = m c^2
\end{equation}

\end{document}
`;
</script>

<script lang="ts">
	import { Button } from '@glyph/ui/button';
	import { Badge } from '@glyph/ui/badge';
	import { ThemeToggle } from '@glyph/ui/theme-toggle';
	import { settings, type LatexGrammar } from '@glyph/ui/settings';
	import CodeEditor from './code-editor.svelte';

	/**
	 * EditorShell — calm editor chrome shared by desktop + web `/editor`.
	 * The source pane is a real CodeMirror surface (JetBrains theme + LaTeX
	 * grammar). Compile/preview wiring (Tectonic via Rust on desktop, WASM on
	 * web) lands next; the preview here is a placeholder.
	 */
	let {
		filename = $bindable('untitled.tex'),
		source = $bindable(SAMPLE_LATEX),
		platform = 'web' as 'web' | 'desktop'
	} = $props();

	const lineCount = $derived(source.split('\n').length);
	const charCount = $derived(source.length);

	const grammars: { id: LatexGrammar; label: string }[] = [
		{ id: 'legacy', label: 'stex' },
		{ id: 'lezer', label: 'lezer' }
	];
</script>

<div class="bg-background text-foreground flex h-dvh flex-col overflow-hidden">
	<!-- Top toolbar -->
	<header
		class="border-border bg-card flex h-11 shrink-0 items-center gap-3 border-b px-3"
	>
		<div class="flex items-center gap-2">
			<div
				class="bg-primary text-primary-foreground grid size-6 place-items-center rounded-md text-[13px] font-semibold"
				aria-hidden="true"
			>
				G
			</div>
			<span class="text-muted-foreground/70 text-sm select-none">Glyph</span>
		</div>

		<div class="bg-border mx-1 h-5 w-px"></div>

		<input
			class="text-foreground placeholder:text-muted-foreground focus-visible:bg-muted/60 w-56 rounded-md bg-transparent px-2 py-1 text-sm outline-none"
			bind:value={filename}
			spellcheck="false"
			aria-label="File name"
		/>

		<Badge variant="default" class="uppercase">LaTeX</Badge>

		<div class="ml-auto flex items-center gap-1.5">
			<!-- LaTeX grammar switch -->
			<div class="border-border bg-muted/40 flex items-center rounded-md border p-0.5" role="group" aria-label="LaTeX grammar">
				{#each grammars as g (g.id)}
					<button
						class="rounded-[5px] px-2 py-1 text-xs font-medium transition-colors {settings.grammar === g.id
							? 'bg-card text-foreground shadow-craft-sm'
							: 'text-muted-foreground hover:text-foreground'}"
						aria-pressed={settings.grammar === g.id}
						onclick={() => (settings.grammar = g.id)}
					>
						{g.label}
					</button>
				{/each}
			</div>

			<Button variant="ghost" size="sm">Format</Button>
			<Button size="sm">Compile</Button>
			<ThemeToggle />
		</div>
	</header>

	<!-- Split: source | preview -->
	<div class="grid min-h-0 flex-1 grid-cols-1 lg:grid-cols-2">
		<!-- Source pane -->
		<section class="flex min-h-0 flex-col">
			<div
				class="text-muted-foreground border-border flex h-8 shrink-0 items-center gap-2 border-b px-3 text-xs"
			>
				<span class="font-medium">Source</span>
				<span class="text-muted-foreground/60">·</span>
				<span class="text-muted-foreground/60">{lineCount} lines</span>
			</div>
			<div class="min-h-0 flex-1">
				<CodeEditor
					bind:value={source}
					theme={settings.resolved}
					grammar={settings.grammar}
					fontSize={settings.fontSize}
					lineWrapping={settings.lineWrapping}
				/>
			</div>
		</section>

		<!-- Preview pane -->
		<section
			class="bg-muted/40 border-border flex min-h-0 flex-col border-t lg:border-t-0 lg:border-l"
		>
			<div
				class="text-muted-foreground border-border flex h-8 shrink-0 items-center gap-2 border-b px-3 text-xs"
			>
				<span class="font-medium">Preview</span>
				<span class="text-muted-foreground/60">·</span>
				<span class="text-muted-foreground/60">live</span>
			</div>
			<div class="min-h-0 flex-1 overflow-auto p-6">
				<div
					class="bg-card text-card-foreground border-border shadow-craft-sm mx-auto max-w-prose rounded-lg border p-8"
				>
					<div
						class="text-muted-foreground/60 mb-4 text-[11px] font-medium tracking-widest uppercase"
					>
						Rendered output
					</div>
					<h1 class="font-display mb-3 text-3xl">Hello from Glyph</h1>
					<p class="text-muted-foreground leading-relaxed">
						The Tectonic (LaTeX) engine renders here — fully offline, in-process.
						Your document never leaves this device.
					</p>
				</div>
			</div>
		</section>
	</div>

	<!-- Status bar -->
	<footer
		class="border-border bg-card text-muted-foreground flex h-7 shrink-0 items-center gap-4 border-t px-3 text-[11px]"
	>
		<span class="flex items-center gap-1.5">
			<span class="bg-success size-1.5 rounded-full"></span>
			Offline-ready
		</span>
		<span>LaTeX · Tectonic</span>
		<span class="text-muted-foreground/60">{settings.grammar === 'legacy' ? 'stex' : 'lezer'}</span>
		<span class="text-muted-foreground/50 ml-auto">Ln {lineCount} · {charCount} chars</span>
		<span class="text-muted-foreground/50 capitalize">{platform}</span>
	</footer>
</div>
