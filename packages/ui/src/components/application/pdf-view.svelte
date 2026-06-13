<script lang="ts">
	import { onMount, onDestroy, tick } from 'svelte';
	import { Button } from '@glyph/ui/button';
	import { IconMinus, IconPlus, IconArrowsMaximize } from '@tabler/icons-svelte';
	// PDF.js' own text-layer styles (selection + span positioning). Scoped under
	// .textLayer / .annotationLayer, so it doesn't touch our design tokens.
	import 'pdfjs-dist/web/pdf_viewer.css';

	/**
	 * PdfView — an in-app PDF renderer built on PDF.js, Overleaf-style:
	 *   - a <canvas> paints each page (HiDPI-crisp), and
	 *   - a transparent **text layer** overlays it so text is selectable/copyable.
	 * No browser PDF chrome; our own fit-to-width scaling + zoom; OS-identical;
	 * zero native libraries. Double-clicking emits the PDF-space click so the host
	 * can reverse-search via SyncTeX.
	 *
	 * pdfjs is imported lazily in onMount so it never runs during SSR/prerender.
	 */
	let {
		data,
		onreverse
	}: {
		data?: Uint8Array;
		onreverse?: (loc: { page: number; x: number; y: number }) => void;
	} = $props();

	/* eslint-disable @typescript-eslint/no-explicit-any */
	let pdfjs: any = null;
	let doc: any = null;
	let renderToken = 0;
	let textLayers: any[] = []; // live TextLayer instances (for cancel)
	let reflowTimer: ReturnType<typeof setTimeout> | undefined;
	let resizeObserver: ResizeObserver | undefined;
	// Guards against redundant re-renders (the main source of flicker): only
	// repaint when the document, fit-width, or zoom actually changed.
	let lastRenderKey = '';

	let pages = $state<number[]>([]);
	let canvasEls = $state<HTMLCanvasElement[]>([]);
	let textEls = $state<HTMLDivElement[]>([]);
	let pageEls = $state<HTMLDivElement[]>([]);
	let surfaceEl = $state<HTMLDivElement>();

	let zoom = $state(1); // 1 = fit width; multiplies the fit scale
	let containerWidth = $state(0);
	let docVersion = $state(0);
	let loading = $state(false);
	let hasRendered = $state(false);
	let errorMsg = $state<string | undefined>(undefined);
	let ready = $state(false);
	let flashEl: HTMLDivElement | null = null;
	let flashTimer: ReturnType<typeof setTimeout> | undefined;

	const SURFACE_PAD = 48; // px-6 on the inner column (24 each side)

	// Per page: viewport used to render + unscaled page height (pt) for sync.
	const meta = new Map<number, { viewport: any; heightPt: number }>();

	onMount(async () => {
		pdfjs = await import('pdfjs-dist');
		const worker = await import('pdfjs-dist/build/pdf.worker.min.mjs?url');
		pdfjs.GlobalWorkerOptions.workerSrc = worker.default;

		if (surfaceEl) {
			containerWidth = surfaceEl.clientWidth;
			resizeObserver = new ResizeObserver((entries) => {
				// Round to whole px so sub-pixel jitter never triggers a re-render.
				const w = Math.round(entries[0].contentRect.width);
				if (w !== containerWidth) containerWidth = w;
			});
			resizeObserver.observe(surfaceEl);
		}

		ready = true;
		await loadDoc();
	});

	onDestroy(() => {
		renderToken++;
		resizeObserver?.disconnect();
		clearTimeout(reflowTimer);
		clearTimeout(flashTimer);
		cancelTextLayers();
		if (doc) {
			try {
				doc.destroy();
			} catch {
				/* ignore */
			}
		}
	});

	// Reload when the PDF bytes change.
	$effect(() => {
		void data;
		if (ready) loadDoc();
	});

	// Re-render (not reload) when fit width or zoom changes.
	$effect(() => {
		void docVersion;
		void containerWidth;
		void zoom;
		if (ready && doc && containerWidth > 0) scheduleRender();
	});

	function cancelTextLayers() {
		for (const tl of textLayers) {
			try {
				tl?.cancel();
			} catch {
				/* ignore */
			}
		}
		textLayers = [];
	}

	function scheduleRender() {
		clearTimeout(reflowTimer);
		reflowTimer = setTimeout(() => renderAll(++renderToken), 60);
	}

	async function loadDoc() {
		if (!pdfjs) return;
		const token = ++renderToken;
		errorMsg = undefined;

		if (!data || data.length === 0) {
			cancelTextLayers();
			if (doc) {
				try {
					doc.destroy();
				} catch {
					/* ignore */
				}
				doc = null;
			}
			pages = [];
			hasRendered = false;
			lastRenderKey = '';
			return;
		}

		if (!hasRendered) loading = true;
		try {
			const bytes = data.slice(); // pdfjs detaches the buffer
			const task = pdfjs.getDocument({ data: bytes });
			const next = await task.promise;
			if (token !== renderToken) {
				next.destroy();
				return;
			}
			cancelTextLayers();
			if (doc) {
				try {
					doc.destroy();
				} catch {
					/* ignore */
				}
			}
			doc = next;
			// Keep existing page elements where possible (same keys ⇒ no remount,
			// old pixels stay on screen until the new paint lands → no white flash).
			if (pages.length !== doc.numPages) {
				pages = Array.from({ length: doc.numPages }, (_, i) => i + 1);
				await tick();
			}
			lastRenderKey = ''; // force a repaint of the new document
			docVersion++; // triggers the render effect
		} catch (e) {
			if (token === renderToken) {
				errorMsg = String(e);
				loading = false;
			}
		}
	}

	async function renderAll(token: number) {
		if (!doc || containerWidth <= 0) return;

		const available = Math.max(120, containerWidth - SURFACE_PAD);
		const key = `${docVersion}:${Math.round(available)}:${zoom}`;
		if (key === lastRenderKey) return; // nothing changed — skip (no flicker)

		const showLoader = !hasRendered;
		if (showLoader) loading = true;
		cancelTextLayers();
		const outputScale = window.devicePixelRatio || 1;
		meta.clear();

		try {
			for (let p = 1; p <= doc.numPages; p++) {
				if (token !== renderToken) return;
				const canvas = canvasEls[p - 1];
				const textEl = textEls[p - 1];
				const pageEl = pageEls[p - 1];
				if (!canvas || !pageEl) continue;

				const page = await doc.getPage(p);
				if (token !== renderToken) return;

				const unscaled = page.getViewport({ scale: 1 });
				const scale = (available / unscaled.width) * zoom;
				const viewport = page.getViewport({ scale });
				meta.set(p, { viewport, heightPt: unscaled.height });

				const w = Math.floor(viewport.width);
				const h = Math.floor(viewport.height);
				pageEl.style.width = `${w}px`;
				pageEl.style.height = `${h}px`;
				pageEl.style.setProperty('--scale-factor', String(scale));

				// Only resize (which clears) the canvas when dimensions truly change;
				// otherwise we repaint over the old frame with no blank flash.
				const cw = Math.floor(viewport.width * outputScale);
				const ch = Math.floor(viewport.height * outputScale);
				if (canvas.width !== cw) canvas.width = cw;
				if (canvas.height !== ch) canvas.height = ch;
				canvas.style.width = `${w}px`;
				canvas.style.height = `${h}px`;

				const ctx = canvas.getContext('2d');
				if (!ctx) continue;
				const transform = outputScale !== 1 ? [outputScale, 0, 0, outputScale, 0, 0] : undefined;
				await page.render({ canvasContext: ctx, viewport, transform }).promise;
				if (token !== renderToken) return;

				// Selectable text overlay.
				if (textEl) {
					textEl.replaceChildren();
					const textLayer = new pdfjs.TextLayer({
						textContentSource: page.streamTextContent(),
						container: textEl,
						viewport
					});
					textLayers.push(textLayer);
					await textLayer.render();
				}
			}
			if (token === renderToken) {
				lastRenderKey = key;
				hasRendered = true;
			}
		} catch (e) {
			if (token === renderToken) errorMsg = String(e);
		} finally {
			if (token === renderToken) loading = false;
		}
	}

	function onDblClick(e: MouseEvent) {
		if (!onreverse) return;
		const pageEl = (e.target as HTMLElement).closest<HTMLElement>('[data-page]');
		if (!pageEl) return;
		const p = parseInt(pageEl.dataset.page ?? '', 10);
		const info = meta.get(p);
		if (!info) return;

		const rect = pageEl.getBoundingClientRect();
		const vx = ((e.clientX - rect.left) / rect.width) * info.viewport.width;
		const vy = ((e.clientY - rect.top) / rect.height) * info.viewport.height;
		const [pdfX, pdfY] = info.viewport.convertToPdfPoint(vx, vy);
		onreverse({ page: p, x: pdfX, y: info.heightPt - pdfY });
	}

	function setZoom(z: number) {
		zoom = Math.min(4, Math.max(0.25, +z.toFixed(2)));
	}

	// Ctrl/Cmd + wheel = zoom (Overleaf/Prism-style); plain scroll still scrolls.
	function onWheel(e: WheelEvent) {
		if (!(e.ctrlKey || e.metaKey)) return;
		e.preventDefault();
		setZoom(zoom - e.deltaY * 0.0015);
	}

	/**
	 * Forward sync: scroll to a PDF region (big-points, `v` from top) and flash a
	 * highlight over it. Called by the host when the caret moves / on shortcut.
	 */
	export function revealLocation(loc: {
		page: number;
		h: number;
		v: number;
		width: number;
		height: number;
		depth: number;
	}) {
		const info = meta.get(loc.page);
		const pageEl = pageEls[loc.page - 1];
		if (!info || !pageEl || !surfaceEl) return;
		const s = info.viewport.scale;
		const topPx = Math.max(0, (loc.v - loc.height) * s);
		const hPx = Math.max(12, (loc.height + loc.depth) * s);
		const leftPx = Math.max(0, loc.h * s);
		const wPx = loc.width > 0 ? loc.width * s : Math.max(24, pageEl.clientWidth - leftPx);

		if (flashEl) flashEl.remove();
		const el = document.createElement('div');
		el.className = 'glyph-sync-flash';
		el.style.left = `${leftPx}px`;
		el.style.top = `${topPx}px`;
		el.style.width = `${wPx}px`;
		el.style.height = `${hPx}px`;
		pageEl.appendChild(el);
		flashEl = el;
		clearTimeout(flashTimer);
		flashTimer = setTimeout(() => {
			el.remove();
			if (flashEl === el) flashEl = null;
		}, 1500);

		const top =
			pageEl.getBoundingClientRect().top -
			surfaceEl.getBoundingClientRect().top +
			surfaceEl.scrollTop +
			topPx -
			100;
		surfaceEl.scrollTo({ top: Math.max(0, top), behavior: 'smooth' });
	}
</script>

<div class="relative flex h-full min-h-0 flex-col">
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		bind:this={surfaceEl}
		class="pdf-surface min-h-0 flex-1 overflow-auto"
		ondblclick={onDblClick}
		onwheel={onWheel}
	>
		{#if errorMsg}
			<div class="border-destructive/30 bg-destructive/5 mx-auto my-6 max-w-prose rounded-lg border p-4">
				<p class="text-destructive text-sm font-medium">Could not display the PDF.</p>
				<pre
					class="text-muted-foreground mt-2 overflow-auto font-mono text-[11px] whitespace-pre-wrap">{errorMsg}</pre>
			</div>
		{:else}
			<div class="flex flex-col items-center gap-4 px-6 py-6">
				{#each pages as p (p)}
					<div
						bind:this={pageEls[p - 1]}
						class="glyph-pdf-page border-border bg-card shadow-craft-sm relative overflow-hidden rounded-sm border"
						data-page={p}
						title="Double-click to jump to the matching source line"
					>
						<canvas bind:this={canvasEls[p - 1]} class="block"></canvas>
						<div bind:this={textEls[p - 1]} class="textLayer"></div>
					</div>
				{/each}
			</div>
		{/if}
	</div>

	<!-- zoom controls (bottom-left so toasts can own the bottom-right corner) -->
	<div
		class="border-border bg-card/90 absolute bottom-4 left-4 flex items-center gap-0.5 rounded-md border p-0.5 shadow-sm backdrop-blur"
	>
		<Button variant="ghost" size="icon-xs" title="Zoom out" onclick={() => setZoom(zoom - 0.1)}>
			<IconMinus />
		</Button>
		<button
			class="text-muted-foreground hover:text-foreground w-12 text-center text-xs tabular-nums transition-colors"
			title="Fit width"
			onclick={() => setZoom(1)}
		>
			{zoom === 1 ? 'Fit' : `${Math.round(zoom * 100)}%`}
		</button>
		<Button variant="ghost" size="icon-xs" title="Zoom in" onclick={() => setZoom(zoom + 0.1)}>
			<IconPlus />
		</Button>
		<Button variant="ghost" size="icon-xs" title="Fit width" onclick={() => setZoom(1)}>
			<IconArrowsMaximize />
		</Button>
	</div>

	{#if loading && !hasRendered}
		<div
			class="text-muted-foreground pointer-events-none absolute inset-0 flex items-center justify-center text-xs"
		>
			Rendering…
		</div>
	{/if}
</div>

<style>
	/* Reserve the scrollbar gutter so the vertical scrollbar appearing never
	   changes the content width — kills the ResizeObserver feedback flicker. */
	.pdf-surface {
		scrollbar-gutter: stable;
	}
	/* Keep the text overlay exactly on the canvas and let selection show through. */
	.glyph-pdf-page :global(.textLayer) {
		position: absolute;
		inset: 0;
	}
	/* Forward-sync flash (source → PDF). Uses the brand token, not a literal. */
	:global(.glyph-sync-flash) {
		position: absolute;
		z-index: 5;
		pointer-events: none;
		border-radius: 3px;
		background: color-mix(in srgb, var(--color-primary) 30%, transparent);
		box-shadow: 0 0 0 1px color-mix(in srgb, var(--color-primary) 45%, transparent);
		animation: glyph-sync-fade 1.5s ease-out forwards;
	}
	@keyframes -global-glyph-sync-fade {
		0% {
			opacity: 0;
		}
		12% {
			opacity: 1;
		}
		70% {
			opacity: 1;
		}
		100% {
			opacity: 0;
		}
	}
</style>
