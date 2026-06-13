<script lang="ts" module>
	export type ViewMode = 'editor' | 'split' | 'preview';

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

	const SAMPLE_BIB = String.raw`@article{glyph2026,
  title   = {Local-first Typesetting},
  author  = {Glyph},
  journal = {Journal of Private Research},
  year    = {2026}
}
`;
</script>

<script lang="ts">
	import { Button } from '@glyph/ui/button';
	import {
	  parseLatexLog,
	  parseSyncTex,
	  summarizeProblems,
	  type SyncTexLocation,
	  type SyncTexMap
	} from '@glyph/ui/editor';
	import { Segmented } from '@glyph/ui/segmented';
	import { ButtonGroup } from '@glyph/ui/button-group';
	import { COMPILE_DEBOUNCE_MS, settings } from '@glyph/ui/settings';
	import { Toaster, toast } from '@glyph/ui/sonner';
	import { ThemeToggle } from '@glyph/ui/theme-toggle';
	import {
	  IconAlertTriangle,
	  IconBolt,
	  IconCurrentLocation,
	  IconEye,
	  IconLayoutColumns,
	  IconLoader2,
	  IconPencil,
	  IconPlayerPlayFilled
	} from '@tabler/icons-svelte';
	import { onDestroy } from 'svelte';

	import ActivityBar, { type ActivityView } from './activity-bar.svelte';
	import CodeEditor from './code-editor.svelte';
	import type { EngineManager } from './engine-settings.svelte';
	import ExportMenu from './export-menu.svelte';
	import FormatToolbar from './format-toolbar.svelte';
	import PdfView from './pdf-view.svelte';
	import ProblemsPanel from './problems-panel.svelte';
	import SidePanel from './side-panel.svelte';

	/**
	 * Workbench — the full editor experience. Top bar (logo · editable title ·
	 * view-modes / theme / export), a left rail + side panel, and the editor /
	 * preview area. Formatting lives in the editor panel header. Calm,
	 * semantic-token-only chrome; the editor keeps its JetBrains identity.
	 */
	let {
		platform = 'web' as 'web' | 'desktop',
		compile,
		engine
	}: {
		platform?: 'web' | 'desktop';
		compile?: (
			source: string
		) => Promise<{ pdf?: string; log?: string; error?: string; synctex?: string }>;
		engine?: EngineManager;
	} = $props();

	type GlyphFile = { id: string; name: string; content: string };

	let files = $state<GlyphFile[]>([
		{ id: 'main', name: 'main.tex', content: SAMPLE_LATEX },
		{
			id: 'intro',
			name: 'sections/introduction.tex',
			content: String.raw`\section{Introduction}

Local-first typesetting keeps your unpublished work on your own machine.
This section motivates the approach.
`
		},
		{
			id: 'results',
			name: 'sections/results.tex',
			content: String.raw`\section{Results}

We observe that $\hat{\theta}$ is consistent, with $\alpha$ scaling as $\beta^2$.
`
		},
		{ id: 'refs', name: 'references.bib', content: SAMPLE_BIB }
	]);
	let activeId = $state('main');
	let source = $state(SAMPLE_LATEX);
	let untitledCount = $state(0);

	const activeFile = $derived(files.find((f) => f.id === activeId) ?? files[0]);

	function persistActive() {
		files = files.map((f) => (f.id === activeId ? { ...f, content: source } : f));
	}
	function openFile(id: string) {
		if (id === activeId) return;
		persistActive();
		activeId = id;
		source = files.find((f) => f.id === id)?.content ?? '';
	}
	function newFile() {
		persistActive();
		untitledCount += 1;
		const id = `untitled-${untitledCount}`;
		files = [...files, { id, name: `untitled-${untitledCount}.tex`, content: '' }];
		activeId = id;
		source = '';
	}
	function renameActive(name: string) {
		files = files.map((f) => (f.id === activeId ? { ...f, name } : f));
	}

	type EditorApi = {
		wrapSelection: (before: string, after?: string) => void;
		insertText: (text: string) => void;
		focusEditor: () => void;
		goToLine: (line: number) => void;
	};
	let editor = $state<EditorApi>();

	let activeView = $state<ActivityView>('files');
	let panelCollapsed = $state(false);
	let viewMode = $state<ViewMode>('split');

	let cursor = $state({ line: 1, column: 1 });

	const lineCount = $derived(source.split('\n').length);
	const charCount = $derived(source.length);
	const wordCount = $derived(source.trim() ? source.trim().split(/\s+/).length : 0);

	// --- Resizable split -------------------------------------------------------
	let splitPct = $state(52);
	let dragging = $state(false);
	let bodyEl = $state<HTMLElement>();

	// --- Resizable sidebar (drag the edge; capped at 20% of the shell width) ---
	let shellEl = $state<HTMLElement>();
	let shellW = $state(1280);
	let sidebarW = $state(240);
	let resizingSidebar = $state(false);
	const ACTIVITY_BAR_PX = 48; // the w-12 rail left of the panel
	const maxSidebar = $derived(Math.max(180, Math.round(shellW * 0.2)));
	const sidebarWidth = $derived(Math.min(sidebarW, maxSidebar));

	$effect(() => {
		if (!shellEl || typeof ResizeObserver === 'undefined') return;
		const ro = new ResizeObserver(() => {
			if (shellEl) shellW = shellEl.getBoundingClientRect().width;
		});
		ro.observe(shellEl);
		return () => ro.disconnect();
	});

	function startResize() {
		if (viewMode === 'split') dragging = true;
	}
	function startSidebarResize() {
		resizingSidebar = true;
	}
	function onPointerMove(e: PointerEvent) {
		if (resizingSidebar && shellEl) {
			const rect = shellEl.getBoundingClientRect();
			const w = e.clientX - rect.left - ACTIVITY_BAR_PX;
			sidebarW = Math.min(maxSidebar, Math.max(180, w));
			return;
		}
		if (!dragging || viewMode !== 'split' || !bodyEl) return;
		const rect = bodyEl.getBoundingClientRect();
		const pct = ((e.clientX - rect.left) / rect.width) * 100;
		splitPct = Math.min(72, Math.max(28, pct));
	}
	function stopResize() {
		dragging = false;
		resizingSidebar = false;
	}

	function selectView(view: ActivityView) {
		if (view === activeView) panelCollapsed = !panelCollapsed;
		else {
			activeView = view;
			panelCollapsed = false;
		}
	}

	function cycleGrammar() {
		settings.grammar = settings.grammar === 'legacy' ? 'lezer' : 'legacy';
	}

	// --- Compilation (Tectonic on desktop; placeholder elsewhere) -------------
	// Live, debounced recompile-as-you-type. Tectonic is a heavier subprocess
	// than a WASM engine, so compiles are *coalesced*: while one is running we
	// never start a second — we mark the doc dirty and recompile once it lands.
	type CompileStatus = 'idle' | 'compiling' | 'success' | 'error';
	let compiling = $state(false);
	let compileStatus = $state<CompileStatus>('idle');
	let lastCompileMs = $state<number | null>(null);
	let pendingRecompile = false;
	let lastCompiledSource: string | null = null;
	let pdfUrl = $state<string | undefined>(undefined);
	let pdfBytes = $state<Uint8Array | undefined>(undefined);
	let synctex = $state<SyncTexMap | undefined>(undefined);
	let compileError = $state<string | undefined>(undefined);
	let compileLog = $state('');

	// Parsed diagnostics for the Problems panel; recomputed on each compile.
	const problems = $derived(parseLatexLog(compileLog, compileError));
	const problemSummary = $derived(summarizeProblems(problems));
	let showProblems = $state(false);

	const canCompile = $derived(Boolean(compile));

	function base64ToBytes(b64: string): Uint8Array {
		const bin = atob(b64);
		const bytes = new Uint8Array(bin.length);
		for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
		return bytes;
	}

	/** Double-click in the PDF → jump the editor to the matching source line. */
	function onReverse(loc: { page: number; x: number; y: number }) {
		const hit = synctex?.locate(loc.page, loc.x, loc.y);
		if (hit) editor?.goToLine(hit.line);
	}

	let pdfView = $state<{ revealLocation: (loc: SyncTexLocation) => void }>();

	/** Forward sync: caret line → scroll/flash the matching region in the PDF. */
	function syncToPdf() {
		if (!synctex) {
			toast.info('Compile first to sync to the PDF.');
			return;
		}
		const loc = synctex.forward(cursor.line);
		if (!loc) {
			toast.info(`No PDF location for line ${cursor.line}.`);
			return;
		}
		if (viewMode === 'editor') viewMode = 'split';
		pdfView?.revealLocation(loc);
	}

	async function runCompile(manual = false) {
		if (!compile) {
			compileError = 'Compilation runs in the Glyph desktop app.';
			compileStatus = 'error';
			if (manual && viewMode === 'editor') viewMode = 'split';
			return;
		}
		// Coalesce: a compile is already in flight — queue exactly one rerun.
		if (compiling) {
			pendingRecompile = true;
			return;
		}
		if (manual && viewMode === 'editor') viewMode = 'split';
		compiling = true;
		try {
			do {
				pendingRecompile = false;
				persistActive();
				const snapshot = source;
				// Skip redundant auto-recompiles of already-rendered content.
				if (!manual && snapshot === lastCompiledSource) break;
				compileStatus = 'compiling';
				const started = performance.now();
				const out = await compile(snapshot);
				lastCompileMs = Math.round(performance.now() - started);
				compileLog = out.log ?? '';
				// Mirror to the devtools console so logs are readable/debuggable.
				if (out.error) {
					console.error(`[Glyph] LaTeX compilation failed (${lastCompileMs}ms): ${out.error}`);
					if (out.log) console.error(out.log);
				} else {
					console.info(`[Glyph] compiled in ${lastCompileMs}ms`);
					if (out.log?.trim()) console.debug(out.log);
				}
				if (out.pdf) {
					if (pdfUrl) URL.revokeObjectURL(pdfUrl);
					const bytes = base64ToBytes(out.pdf);
					pdfBytes = bytes;
					pdfUrl = URL.createObjectURL(
						new Blob([bytes as BlobPart], { type: 'application/pdf' })
					);
					synctex = out.synctex ? parseSyncTex(out.synctex) : undefined;
					compileError = undefined;
					compileStatus = 'success';
					lastCompiledSource = snapshot;
				} else {
					compileError = out.error ?? 'Compilation failed.';
					compileStatus = 'error';
					showProblems = true; // surface failures immediately
				}
			} while (pendingRecompile);
		} catch (e) {
			compileError = String(e);
			compileStatus = 'error';
			showProblems = true;
			console.error('[Glyph] compile threw:', e);
		} finally {
			compiling = false;
		}
	}

	// Debounced auto-compile: re-arm on every edit (and on file switch, since
	// `source` changes). Disabled when the user turns auto-compile off or no
	// engine is wired (web build leaves the prop unset when desktop-only).
	$effect(() => {
		void source; // track edits + file switches
		const auto = settings.autoCompile;
		if (!compile || !auto) return;
		const timer = setTimeout(() => runCompile(false), COMPILE_DEBOUNCE_MS);
		return () => clearTimeout(timer);
	});

	const compileLabel = $derived(
		!canCompile
			? 'Desktop only'
			: compileStatus === 'compiling'
				? 'Compiling…'
				: compileStatus === 'error'
					? 'Compile error'
					: compileStatus === 'success'
						? lastCompileMs != null
							? `Compiled in ${(lastCompileMs / 1000).toFixed(1)}s`
							: 'Compiled'
						: 'Ready'
	);

	function onKeydown(e: KeyboardEvent) {
		if (!(e.ctrlKey || e.metaKey)) return;
		// Ctrl/Cmd+S or Ctrl/Cmd+Enter forces an immediate compile.
		if (e.key === 's' || e.key === 'Enter') {
			e.preventDefault();
			runCompile(true);
		}
		// Ctrl/Cmd+J jumps the PDF to the caret's line (forward sync).
		if (e.key === 'j' || e.key === 'J') {
			e.preventDefault();
			syncToPdf();
		}
	}

	onDestroy(() => {
		if (pdfUrl) URL.revokeObjectURL(pdfUrl);
	});
</script>

<svelte:window onpointermove={onPointerMove} onpointerup={stopResize} onkeydown={onKeydown} />

<div class="bg-background text-foreground flex h-dvh flex-col overflow-hidden">
	<!-- Top bar -->
	<header class="border-border bg-card flex h-11 shrink-0 items-center gap-3 border-b px-3">
		<!-- Logo -->
		<a href="/" class="flex items-center gap-2" aria-label="Glyph home">
			<span
				class="bg-primary text-primary-foreground grid size-6 place-items-center rounded-md text-[13px] font-semibold"
				aria-hidden="true">G</span
			>
			<span class="font-display hidden text-sm sm:inline">Glyph</span>
		</a>

		<!-- Editable filename (centre) -->
		<div class="flex flex-1 justify-center">
			<input
				class="text-foreground placeholder:text-muted-foreground hover:bg-muted/40 focus-visible:bg-muted/60 w-64 rounded-md bg-transparent px-2 py-1 text-center text-sm font-medium outline-none transition-colors"
				value={activeFile.name}
				oninput={(e) => renameActive(e.currentTarget.value)}
				spellcheck="false"
				aria-label="Document title"
			/>
		</div>

		<!-- Right cluster -->
		<div class="flex items-center gap-2">
			{#snippet miEditor()}<IconPencil class="size-3.5" />{/snippet}
			{#snippet miSplit()}<IconLayoutColumns class="size-3.5" />{/snippet}
			{#snippet miPreview()}<IconEye class="size-3.5" />{/snippet}
			<Segmented
			size="sm"
				fill={false}
				aria-label="View mode"
				value={viewMode}
				onValueChange={(v) => (viewMode = v as ViewMode)}
				options={[
					{ value: 'editor', icon: miEditor, title: 'Editor' },
					{ value: 'split', icon: miSplit, title: 'Split' },
					{ value: 'preview', icon: miPreview, title: 'Preview' }
				]}
			/>

			<ButtonGroup>
				<!-- Auto-compile toggle -->
			<Button
				variant={settings.autoCompile ? 'secondary' : 'ghost'}
				size="icon-sm"
				title={settings.autoCompile ? 'Auto-compile on (live)' : 'Auto-compile off'}
				aria-label="Toggle auto-compile"
				aria-pressed={settings.autoCompile}
				onclick={() => (settings.autoCompile = !settings.autoCompile)}
			>
				<IconBolt class={settings.autoCompile ? '' : 'opacity-50'} />
			</Button>

			<Button  onclick={() => runCompile(true)} disabled={compiling} size="sm">
				{#if compiling}
					<IconLoader2 class="animate-spin" />
				{:else}
					<IconPlayerPlayFilled />
				{/if}
				{compiling ? 'Compiling…' : 'Compile'}
			</Button>
			</ButtonGroup>
			<ThemeToggle size="icon-sm" />
			<ExportMenu {source} filename={activeFile.name} {pdfUrl} size="xs" />
		</div>
	</header>

	<!-- Body -->
	<div bind:this={shellEl} class="flex min-h-0 flex-1">
		<ActivityBar active={activeView} onselect={selectView} />

		<!-- Smooth collapse + drag-to-resize (panel stays mounted; capped at 20%). -->
		<div
			class="shrink-0 overflow-hidden {resizingSidebar
				? ''
				: 'transition-[width] duration-300 ease-[cubic-bezier(0.625,0.05,0,1)]'} {panelCollapsed
				? 'pointer-events-none'
				: ''}"
			style:width={panelCollapsed ? '0px' : `${sidebarWidth}px`}
			aria-hidden={panelCollapsed}
		>
			<SidePanel
				view={activeView}
				{files}
				{activeId}
				{engine}
				widthPx={sidebarWidth}
				onopen={openFile}
				onnew={newFile}
			/>
		</div>

		{#if !panelCollapsed}
			<div
				class="group relative z-10 flex w-1 shrink-0 cursor-col-resize touch-none items-center justify-center"
				role="separator"
				aria-orientation="vertical"
				aria-label="Resize sidebar"
				tabindex="-1"
				onpointerdown={startSidebarResize}
			>
				<span
					class="h-10 w-0.5 rounded-full transition-colors {resizingSidebar
						? 'bg-primary'
						: 'bg-border group-hover:bg-primary/60'}"
				></span>
			</div>
		{/if}

		<main class="flex min-h-0 flex-1 flex-col">
			<div bind:this={bodyEl} class="flex min-h-0 flex-1">
				{#if viewMode !== 'preview'}
					<section
						class="flex min-h-0 shrink-0 flex-col {viewMode === 'split'
							? 'border-border border-r'
							: ''}"
						style={viewMode === 'split' ? `width:${splitPct}%` : 'width:100%'}
					>
						<div
							class="text-muted-foreground border-border flex h-9 shrink-0 items-center gap-2 border-b px-2 text-xs"
						>
							<span class="px-1 font-medium">Source</span>
							<span class="text-muted-foreground/60">{lineCount} lines</span>
							<div class="ml-auto flex items-center gap-1">
								<button
									class="hover:bg-muted hover:text-foreground grid size-6 place-items-center rounded transition-colors"
									title="Sync to PDF (⌘/Ctrl+J)"
									aria-label="Sync to PDF"
									onclick={syncToPdf}
								>
									<IconCurrentLocation size={15} />
								</button>
								<FormatToolbar
									wrap={(b, a) => editor?.wrapSelection(b, a)}
									insert={(t) => editor?.insertText(t)}
								/>
							</div>
						</div>
						<div class="min-h-0 flex-1">
							<CodeEditor
								bind:this={editor}
								bind:value={source}
								theme={settings.resolved}
								grammar={settings.grammar}
								fontSize={settings.fontSize}
								fontFamily={settings.fontStack}
								lineWrapping={settings.lineWrapping}
								oncursor={(p) => (cursor = p)}
							/>
						</div>
					</section>
				{/if}

				{#if viewMode === 'split'}
					<div
						class="group relative z-10 flex w-1 shrink-0 cursor-col-resize touch-none items-center justify-center"
						role="separator"
						aria-orientation="vertical"
						aria-valuenow={Math.round(splitPct)}
						tabindex="-1"
						onpointerdown={startResize}
					>
						<span
							class="h-10 w-0.5 rounded-full transition-colors {dragging
								? 'bg-primary'
								: 'bg-border group-hover:bg-primary/60'}"
						></span>
					</div>
				{/if}

				{#if viewMode !== 'editor'}
					<section class="bg-muted/40 flex min-h-0 min-w-0 flex-1 flex-col">
						<div
							class="text-muted-foreground border-border flex h-9 shrink-0 items-center gap-2 border-b px-3 text-xs"
						>
							<span class="font-medium">Preview</span>
							<span class="text-muted-foreground/60">·</span>
							<span
								class="inline-flex items-center gap-1.5 {compileStatus === 'error'
									? 'text-destructive'
									: 'text-muted-foreground/70'}"
							>
								{#if compileStatus === 'compiling'}
									<IconLoader2 size={12} class="animate-spin" />
								{:else if compileStatus === 'error'}
									<IconAlertTriangle size={12} />
								{:else}
									<span
										class="size-1.5 rounded-full {compileStatus === 'success'
											? 'bg-success'
											: 'bg-muted-foreground/40'}"
									></span>
								{/if}
								{compileLabel}
							</span>
							{#if pdfBytes && synctex}
								<span class="text-muted-foreground/50 ml-auto hidden sm:inline">
									double-click → source
								</span>
							{/if}
						</div>
						<div class="min-h-0 flex-1">
							{#if pdfBytes}
								<PdfView bind:this={pdfView} data={pdfBytes} onreverse={onReverse} />
							{:else}
								<div class="h-full overflow-auto p-6">
									{#if compileError}
										<div
											class="border-destructive/30 bg-destructive/5 mx-auto max-w-prose rounded-lg border p-4"
										>
											<p class="text-destructive text-sm font-medium">{compileError}</p>
											{#if compileLog}
												<pre
													class="text-muted-foreground mt-3 max-h-72 overflow-auto font-mono text-[11px] whitespace-pre-wrap">{compileLog}</pre>
											{/if}
										</div>
									{:else}
										<div
											class="glyph-print-area bg-card text-card-foreground border-border shadow-craft-sm mx-auto max-w-prose rounded-lg border p-8"
										>
											<div
												class="text-muted-foreground/60 mb-4 text-[11px] font-medium tracking-widest uppercase"
											>
												Rendered output
											</div>
											<h1 class="font-display mb-3 text-3xl">Hello from Glyph</h1>
											<p class="text-muted-foreground leading-relaxed">
												{#if canCompile}
													{settings.autoCompile
														? 'Start typing — Glyph recompiles live with Tectonic, fully offline. Your document never leaves this device.'
														: 'Press Compile (or ⌘/Ctrl+S) to render with Tectonic — fully offline, in-process.'}
												{:else}
													Compilation runs in the Glyph desktop app — fully offline, in-process.
												{/if}
											</p>
										</div>
									{/if}
								</div>
							{/if}
						</div>
					</section>
				{/if}
			</div>

			{#if showProblems}
				<ProblemsPanel
					{problems}
					log={compileLog}
					ongoto={(l) => {
						editor?.goToLine(l);
						if (viewMode === 'preview') viewMode = 'split';
					}}
					onclose={() => (showProblems = false)}
				/>
			{/if}

			<!-- Status bar -->
			<footer
				class="border-border bg-card text-muted-foreground flex h-7 shrink-0 items-center gap-4 border-t px-3 text-[11px]"
			>
				<span
					class="flex items-center gap-1.5 {compileStatus === 'error' ? 'text-destructive' : ''}"
				>
					<span
						class="size-1.5 rounded-full {compileStatus === 'compiling'
							? 'bg-primary animate-pulse'
							: compileStatus === 'error'
								? 'bg-destructive'
								: 'bg-success'}"
					></span>
					{compileLabel}
				</span>
				<span class="text-muted-foreground/50">·</span>
				<span>{settings.autoCompile ? 'Auto' : 'Manual'}</span>
				<span>LaTeX · Tectonic</span>
				<button
					class="hover:text-foreground transition-colors"
					title="Toggle LaTeX grammar"
					onclick={cycleGrammar}
				>
					{settings.grammar === 'legacy' ? 'stex' : 'lezer'}
				</button>
				<button
					class="inline-flex items-center gap-1 transition-colors hover:text-foreground {problemSummary.errors
						? 'text-destructive'
						: problemSummary.warnings
							? 'text-warning'
							: ''}"
					title="Toggle problems / log"
					aria-pressed={showProblems}
					onclick={() => (showProblems = !showProblems)}
				>
					<IconAlertTriangle size={12} />
					<span class="tabular-nums">{problemSummary.errors}/{problemSummary.warnings}</span>
				</button>
				<span class="text-muted-foreground/50 ml-auto"
					>Ln {cursor.line}, Col {cursor.column}</span
				>
				<span class="text-muted-foreground/50">{wordCount} words · {charCount} chars</span>
				<span class="text-muted-foreground/50 capitalize">{platform}</span>
			</footer>
		</main>
	</div>
</div>

<!-- Toast feedback (bottom-right; matches the app's corner-notification language). -->
<Toaster />

<style>
	/* PDF export = print the preview page only, until Tectonic compiles for real. */
	@media print {
		:global(body *) {
			visibility: hidden !important;
		}
		:global(.glyph-print-area),
		:global(.glyph-print-area *) {
			visibility: visible !important;
		}
		:global(.glyph-print-area) {
			position: fixed;
			inset: 0;
			max-width: none;
			border: none;
			overflow: visible;
		}
	}
</style>
