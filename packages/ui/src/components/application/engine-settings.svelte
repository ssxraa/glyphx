<script lang="ts" module>
	export type EngineVersion = {
		version: string;
		tag: string;
		installed: boolean;
		active: boolean;
	};

	export type CacheInfo = { path: string; bytes: number };
	export type PrefetchResult = { success: boolean; message?: string };

	/** Which System TeX tooling is available on PATH. */
	export type SystemTexInfo = {
		latexmk: boolean;
		pdflatex: boolean;
		xelatex: boolean;
		lualatex: boolean;
		version?: string;
	};

	/** Provided by the host (desktop) — backed by Tauri commands. */
	export type EngineManager = {
		label: string;
		list: () => Promise<EngineVersion[]>;
		download: (version: string) => Promise<string>;
		setActive: (version: string) => Promise<void>;
		/** Uninstall a downloaded version to reclaim disk (optional). */
		remove?: (version: string) => Promise<void>;
		/** Detect a local TeX install for the System TeX engine (optional). */
		detectSystem?: () => Promise<SystemTexInfo>;
		/** Managed package-cache controls (optional). */
		cacheInfo?: () => Promise<CacheInfo>;
		clearCache?: () => Promise<void>;
		prefetch?: () => Promise<PrefetchResult>;
	};
</script>

<script lang="ts">
	import { Button } from '@glyphx/ui/button';
	import { Segmented } from '@glyphx/ui/segmented';
	import {
		settings,
		TEX_PROGRAM_LABELS,
		type EngineKind,
		type TexProgram
	} from '@glyphx/ui/settings';
	import { toast } from '@glyphx/ui/sonner';

	/**
	 * EngineSettings — pick the compile engine (bundled Tectonic or a local
	 * System TeX install), list / download / activate / remove Tectonic versions
	 * from GitHub releases (no app rebuild), and manage the package cache.
	 */
	let { engine }: { engine: EngineManager } = $props();

	const kindOpts: { value: EngineKind; label: string }[] = [
		{ value: 'tectonic', label: 'Tectonic' },
		{ value: 'system', label: 'System TeX' }
	];

	// Detected local TeX tooling (for the System TeX engine).
	let sysInfo = $state<SystemTexInfo | undefined>(undefined);
	let detecting = $state(false);

	async function detectSystem() {
		if (!engine.detectSystem) return;
		detecting = true;
		try {
			sysInfo = await engine.detectSystem();
		} catch {
			/* ignore — UI just shows "not detected" */
		} finally {
			detecting = false;
		}
	}

	// Auto-detect the local TeX install the first time the System TeX view shows.
	function autoDetect() {
		if (!sysInfo && !detecting) detectSystem();
	}

	// Auto-fetch the Tectonic version list the first time that view shows.
	function autoCheck() {
		if (!loaded && !loading) refresh();
	}

	/**
	 * Svelte action: run `callback` once, when `node` first enters the viewport.
	 * Lets each engine view lazily load (versions / detection) on scroll-in
	 * instead of requiring a manual click.
	 */
	function onVisible(node: HTMLElement, callback: () => void) {
		const io = new IntersectionObserver(
			(entries) => {
				if (entries.some((e) => e.isIntersecting)) {
					callback();
					io.disconnect();
				}
			},
			{ threshold: 0.01 }
		);
		io.observe(node);
		return { destroy: () => io.disconnect() };
	}

	// Which TeX programs are available, for the program picker.
	const programOpts = $derived(
		(['pdflatex', 'xelatex', 'lualatex'] as TexProgram[])
			.filter((p) => sysInfo?.[p])
			.map((p) => ({ value: p, label: TEX_PROGRAM_LABELS[p] }))
	);
	const systemReady = $derived(!!sysInfo?.latexmk);

	let versions = $state<EngineVersion[]>([]);
	let loading = $state(false);
	let busy = $state<string | undefined>(undefined);
	let error = $state<string | undefined>(undefined);
	let loaded = $state(false);
	let showAll = $state(false);

	const PREVIEW = 8;
	const shown = $derived(showAll ? versions : versions.slice(0, PREVIEW));

	let cache = $state<CacheInfo | undefined>(undefined);
	let prefetching = $state(false);
	let clearing = $state(false);

	function fmtBytes(n: number): string {
		if (!n) return '0 B';
		const u = ['B', 'KB', 'MB', 'GB'];
		const i = Math.min(u.length - 1, Math.floor(Math.log(n) / Math.log(1024)));
		return `${(n / 1024 ** i).toFixed(i ? 1 : 0)} ${u[i]}`;
	}

	async function refreshCache() {
		if (!engine.cacheInfo) return;
		try {
			cache = await engine.cacheInfo();
		} catch {
			/* ignore */
		}
	}

	async function prefetch() {
		if (!engine.prefetch) return;
		prefetching = true;
		try {
			const r = await engine.prefetch();
			if (r.success) toast.success('Common packages cached for offline use');
			else toast.error(r.message ?? 'Prefetch failed');
			await refreshCache();
		} catch (e) {
			toast.error(String(e));
		} finally {
			prefetching = false;
		}
	}

	async function clearCache() {
		if (!engine.clearCache) return;
		clearing = true;
		try {
			await engine.clearCache();
			toast.success('Package cache cleared');
			await refreshCache();
		} catch (e) {
			toast.error(String(e));
		} finally {
			clearing = false;
		}
	}

	$effect(() => {
		refreshCache();
	});

	async function refresh() {
		loading = true;
		error = undefined;
		try {
			versions = await engine.list();
			loaded = true;
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function download(version: string) {
		busy = version;
		error = undefined;
		try {
			await engine.download(version);
			await refresh();
		} catch (e) {
			error = String(e);
		} finally {
			busy = undefined;
		}
	}

	async function use(version: string) {
		error = undefined;
		try {
			await engine.setActive(version);
			await refresh();
		} catch (e) {
			error = String(e);
		}
	}

	async function remove(version: string) {
		if (!engine.remove) return;
		busy = version;
		error = undefined;
		try {
			await engine.remove(version);
			await refresh();
		} catch (e) {
			error = String(e);
		} finally {
			busy = undefined;
		}
	}
</script>

<div class="flex flex-col gap-3">
	<!-- Which engine compiles: bundled Tectonic, or a local System TeX install. -->
	<div class="flex flex-col gap-1.5">
		<Segmented
			options={kindOpts}
			value={settings.engineKind}
			onValueChange={(v) => (settings.engineKind = v as EngineKind)}
			size="sm"
			aria-label="Compile engine"
		/>
		<p class="text-muted-foreground text-[11px] leading-relaxed">
			{#if settings.engineKind === 'system'}
				Your local TeX install, via latexmk.
			{:else}
				Bundled LaTeX — no setup, packages fetched on demand.
			{/if}
		</p>
	</div>

	{#if settings.engineKind === 'system'}
		<!-- System TeX: detection + program picker. -->
		<div class="flex flex-col gap-2" use:onVisible={autoDetect}>
			{#if detecting}
				<p class="text-muted-foreground text-xs">Looking for a TeX installation…</p>
			{:else if systemReady}
				<div class="flex items-center gap-2">
					<span class="text-success text-[10px] font-semibold tracking-wide uppercase">Detected</span>
					{#if sysInfo?.version}
						<span class="text-muted-foreground/80 truncate text-xs">{sysInfo.version}</span>
					{/if}
				</div>
				{#if programOpts.length}
					<div class="flex flex-col gap-1">
						<span class="text-muted-foreground text-[10px] font-medium tracking-wide uppercase">
							TeX program
						</span>
						<Segmented
							options={programOpts}
							value={settings.texProgram}
							onValueChange={(v) => (settings.texProgram = v as TexProgram)}
							size="sm"
							aria-label="TeX program"
						/>
					</div>
				{/if}
			{:else}
				<div class="border-border text-muted-foreground rounded-md border border-dashed px-3 py-2.5 text-xs leading-relaxed">
					No TeX installation found on your PATH. Install <span class="text-foreground">TeX Live</span>
					or <span class="text-foreground">MiKTeX</span> (both free, cross-platform), then re-check.
					<Button variant="outline" size="xs" class="mt-2 self-start" onclick={detectSystem}>
						Re-check
					</Button>
				</div>
			{/if}
		</div>
	{:else}
	<div class="flex items-center justify-between" use:onVisible={autoCheck}>
		<span class="text-muted-foreground text-[10px] font-medium tracking-wide uppercase">
			{engine.label} engine
		</span>
		<Button variant="ghost" size="xs" onclick={refresh} disabled={loading}>
			{loading ? 'Checking…' : 'Refresh'}
		</Button>
	</div>

	{#if error}
		<p class="text-destructive text-xs">{error}</p>
	{/if}

	{#if loading}
		<p class="text-muted-foreground text-xs">Checking available versions…</p>
	{/if}

	{#if versions.length}
		<div class="flex flex-col gap-1">
			{#each shown as v (v.version)}
				{@const nightly = v.version === 'nightly'}
				<div class="border-border flex items-center gap-2 rounded-md border px-2 py-1.5">
					<span
						class="text-foreground shrink-0 text-[13px] {nightly ? 'capitalize' : 'tabular-nums'}"
						title={nightly
							? 'Newer xetex-layout — fixes the fontawesome5 / icon-font crash'
							: undefined}
					>
						{v.version}
					</span>
					{#if v.active}
						<span class="text-success shrink-0 text-[10px] font-semibold tracking-wide uppercase">
							Active
						</span>
					{:else if v.installed}
						<span
							class="text-muted-foreground/60 shrink-0 text-[10px] font-medium tracking-wide uppercase"
						>
							Installed
						</span>
					{/if}
					<div class="ml-auto flex shrink-0 items-center gap-1">
						{#if !v.installed}
							<Button
								variant="outline"
								size="xs"
								onclick={() => download(v.version)}
								disabled={busy === v.version}
							>
								{busy === v.version ? 'Downloading…' : 'Download'}
							</Button>
						{:else}
							{#if !v.active}
								<Button variant="secondary" size="xs" onclick={() => use(v.version)}>Use</Button>
							{/if}
							{#if engine.remove}
								<Button
									variant="ghost"
									size="xs"
									class="text-muted-foreground hover:text-destructive"
									onclick={() => remove(v.version)}
									disabled={busy === v.version}
									title="Uninstall this version"
								>
									{busy === v.version ? 'Removing…' : 'Remove'}
								</Button>
							{/if}
						{/if}
					</div>
				</div>
			{/each}
			{#if versions.length > PREVIEW}
				<Button
					variant="ghost"
					size="xs"
					class="text-muted-foreground self-start"
					onclick={() => (showAll = !showAll)}
				>
					{showAll ? 'Show fewer' : `Show all ${versions.length} versions`}
				</Button>
			{/if}
		</div>
	{/if}

	{#if engine.cacheInfo}
		<div class="border-border/60 mt-1 flex flex-col gap-1.5 border-t pt-2">
			<div class="flex items-center justify-between">
				<span class="text-muted-foreground text-[10px] font-medium tracking-wide uppercase">
					Package cache
				</span>
				{#if cache}
					<span class="text-muted-foreground/70 text-xs tabular-nums">{fmtBytes(cache.bytes)}</span>
				{/if}
			</div>
			<div class="flex gap-1.5">
				{#if engine.prefetch}
					<Button variant="outline" size="xs" onclick={prefetch} disabled={prefetching}>
						{prefetching ? 'Caching…' : 'Prefetch common'}
					</Button>
				{/if}
				{#if engine.clearCache}
					<Button variant="ghost" size="xs" onclick={clearCache} disabled={clearing}>
						{clearing ? 'Clearing…' : 'Clear cache'}
					</Button>
				{/if}
			</div>
		</div>
	{/if}
	{/if}
</div>
