<script lang="ts" module>
	export type EngineVersion = {
		version: string;
		tag: string;
		installed: boolean;
		active: boolean;
	};

	/** Provided by the host (desktop) — backed by Tauri commands. */
	export type EngineManager = {
		label: string;
		list: () => Promise<EngineVersion[]>;
		download: (version: string) => Promise<string>;
		setActive: (version: string) => Promise<void>;
	};
</script>

<script lang="ts">
	import { Button } from '@glyph/ui/button';

	/**
	 * EngineSettings — list / download / activate engine (Tectonic) versions from
	 * GitHub releases, so the engine can be updated without rebuilding the app.
	 */
	let { engine }: { engine: EngineManager } = $props();

	let versions = $state<EngineVersion[]>([]);
	let loading = $state(false);
	let busy = $state<string | undefined>(undefined);
	let error = $state<string | undefined>(undefined);
	let loaded = $state(false);

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
</script>

<div class="flex flex-col gap-2">
	<div class="flex items-center justify-between">
		<span class="text-muted-foreground text-xs font-medium tracking-wide uppercase">
			{engine.label} engine
		</span>
		<Button variant="ghost" size="sm" onclick={refresh} disabled={loading}>
			{loading ? 'Checking…' : 'Check versions'}
		</Button>
	</div>

	{#if error}
		<p class="text-destructive text-xs">{error}</p>
	{/if}

	{#if !loaded && !loading}
		<p class="text-muted-foreground text-xs">
			Fetch available Tectonic versions to download or switch — no rebuild needed.
		</p>
	{/if}

	{#if versions.length}
		<div class="flex flex-col gap-1">
			{#each versions.slice(0, 8) as v (v.version)}
				<div class="border-border flex items-center gap-2 rounded-md border px-2 py-1.5">
					<span class="text-foreground text-sm tabular-nums">{v.version}</span>
					{#if v.active}
						<span class="text-success text-[10px] font-semibold tracking-wide uppercase">Active</span>
					{:else if v.installed}
						<span class="text-muted-foreground/70 text-[10px] tracking-wide uppercase">Installed</span>
					{/if}
					<div class="ml-auto">
						{#if !v.installed}
							<Button
								variant="outline"
								size="xs"
								onclick={() => download(v.version)}
								disabled={busy === v.version}
							>
								{busy === v.version ? 'Downloading…' : 'Download'}
							</Button>
						{:else if !v.active}
							<Button variant="secondary" size="xs" onclick={() => use(v.version)}>Use</Button>
						{/if}
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>
