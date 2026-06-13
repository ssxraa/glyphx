<script lang="ts" module>
	import type { Project } from '@glyphx/ui/projects';
	export type { Project };
</script>

<script lang="ts">
	import { Button } from '@glyphx/ui/button';
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuSeparator,
		DropdownMenuTrigger
	} from '@glyphx/ui/dropdown-menu';
	import { Logo } from '@glyphx/ui/logo';
	import { projectViewTransitionName } from '@glyphx/ui/projects';
	import { ThemeToggle } from '@glyphx/ui/theme-toggle';
	import {
		IconCopy,
		IconDotsVertical,
		IconFileImport,
		IconFolderOpen,
		IconPencil,
		IconPlus,
		IconSearch,
		IconSettings,
		IconTrash
	} from '@tabler/icons-svelte';

	/**
	 * ProjectsHome — the application's home screen: every LaTeX project listed as
	 * a document-style card. Calm monochrome chrome; the cards themselves read as
	 * little pages. The host owns the data (the projects store) and the actions.
	 * Folder actions (`onopenfolder` / `onimport`) appear only when the host
	 * supports them (desktop); on web only `oncreate` is shown.
	 */
	let {
		projects = [],
		oncreate,
		onopenfolder,
		onimport,
		onopen,
		onrename,
		onduplicate,
		ondelete,
		onsettings
	}: {
		projects?: Project[];
		oncreate?: () => void;
		/** Open an existing project folder from disk (desktop). */
		onopenfolder?: () => void;
		/** Import a .zip project (desktop). */
		onimport?: () => void;
		onopen?: (id: string) => void;
		onrename?: (id: string, name: string) => void;
		onduplicate?: (id: string) => void;
		ondelete?: (id: string) => void;
		/** Open the app settings page (desktop). */
		onsettings?: () => void;
	} = $props();

	let query = $state('');
	let renaming = $state<string | null>(null);
	let renameValue = $state('');

	const filtered = $derived(
		projects.filter((p) => p.name.toLowerCase().includes(query.trim().toLowerCase()))
	);

	function relativeTime(ts: number): string {
		const diff = Date.now() - ts;
		const min = Math.round(diff / 60_000);
		if (min < 1) return 'just now';
		if (min < 60) return `${min} min ago`;
		const hr = Math.round(min / 60);
		if (hr < 24) return `${hr} hr ago`;
		const day = Math.round(hr / 24);
		if (day < 30) return `${day} day${day === 1 ? '' : 's'} ago`;
		const mo = Math.round(day / 30);
		return `${mo} mo ago`;
	}

	function startRename(p: Project) {
		renaming = p.id;
		renameValue = p.name;
	}
	function commitRename(id: string) {
		const name = renameValue.trim();
		if (name) onrename?.(id, name);
		renaming = null;
	}

	// Deterministic-but-varied faux text lines so each card's "page" looks unique.
	function lineWidths(seed: string, n = 5): number[] {
		let h = 0;
		for (let i = 0; i < seed.length; i++) h = (h * 31 + seed.charCodeAt(i)) >>> 0;
		return Array.from({ length: n }, (_, i) => {
			h = (h * 1103515245 + 12345) >>> 0;
			return 55 + ((h >> (i + 3)) % 42); // 55–96%
		});
	}

</script>

<div class="bg-background text-foreground flex h-dvh flex-col overflow-hidden">
	<!-- Top bar -->
	<header class="border-border flex h-14 shrink-0 items-center gap-3 border-b px-5">
		<Logo size={26} viewTransitionName="app-logo" class="text-base tracking-tight" />
		<div class="ml-auto flex items-center gap-2">
			<ThemeToggle size="icon-sm" />
			{#if onsettings}
				<Button
					size="icon-sm"
					variant="ghost"
					title="Settings"
					aria-label="Settings"
					onclick={() => onsettings?.()}
				>
					<IconSettings size={16} />
				</Button>
			{/if}
			{#if onimport}
				<Button size="sm" variant="outline" onclick={() => onimport?.()}>
					<IconFileImport size={15} />
					Import
				</Button>
			{/if}
			{#if onopenfolder}
				<Button size="sm" variant="outline" onclick={() => onopenfolder?.()}>
					<IconFolderOpen size={15} />
					Open folder
				</Button>
			{/if}
			<Button size="sm" onclick={() => oncreate?.()}>
				<IconPlus size={15} />
				New project
			</Button>
		</div>
	</header>

	<div class="min-h-0 flex-1 overflow-auto">
		<div class="mx-auto w-full max-w-[1100px] px-6 py-8">
			<!-- Title row + search -->
			<div class="mb-6 flex flex-wrap items-end justify-between gap-4">
				<div>
					<h1 class="font-display text-2xl tracking-tight">Projects</h1>
					<p class="text-muted-foreground mt-1 text-sm">
						{projects.length}
						{projects.length === 1 ? 'project' : 'projects'} · stored on this device
					</p>
				</div>
				<div class="relative">
					<IconSearch
						size={15}
						class="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2"
					/>
					<input
						bind:value={query}
						class="bg-card border-border text-foreground placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/40 h-9 w-60 rounded-lg border py-1.5 pr-3 pl-8 text-sm outline-none transition-[box-shadow,border-color] focus-visible:ring-2"
						placeholder="Search projects"
						spellcheck="false"
						aria-label="Search projects"
					/>
				</div>
			</div>

			{#if filtered.length === 0}
				<div
					class="border-border text-muted-foreground mt-10 flex flex-col items-center gap-4 rounded-2xl border border-dashed py-20 text-center"
				>
					<Logo text={false} badge={false} size={44} class="opacity-40" />
					{#if projects.length === 0}
						<p class="text-foreground text-sm font-medium">No projects yet</p>
						<p class="max-w-xs text-xs leading-relaxed">
							Create your first project — everything stays on this device.
						</p>
						<Button size="sm" class="mt-1" onclick={() => oncreate?.()}>
							<IconPlus size={15} /> New project
						</Button>
					{:else}
						<p class="text-sm">No projects match “{query}”.</p>
					{/if}
				</div>
			{:else}
				<div
					class="grid grid-cols-[repeat(auto-fill,minmax(190px,1fr))] gap-5"
					role="list"
					aria-label="Projects"
				>
					<!-- New-project card -->
					<button
						class="group border-border hover:border-ring/50 hover:bg-muted/30 flex aspect-[4/5] flex-col items-center justify-center gap-2 rounded-xl border border-dashed transition-[colors,transform,box-shadow] duration-200 ease-out hover:-translate-y-1 active:translate-y-0 active:scale-[0.99] motion-reduce:transform-none"
						onclick={() => oncreate?.()}
					>
						<span
							class="bg-muted text-muted-foreground group-hover:bg-brand-subtle group-hover:text-brand grid size-10 place-items-center rounded-full transition-colors"
						>
							<IconPlus size={20} />
						</span>
						<span class="text-muted-foreground group-hover:text-foreground text-sm font-medium">
							New project
						</span>
					</button>

					{#each filtered as p (p.id)}
						<div class="group relative" role="listitem">
							<button
								class="block w-full text-left"
								onclick={() => onopen?.(p.id)}
								aria-label={`Open ${p.name}`}
							>
								<!-- The "document" — a little page. Shares a view-transition-name
								     with the editor surface so it morphs into the editor. -->
								<div
									class="bg-card border-border shadow-craft-sm group-hover:shadow-craft-md relative aspect-[4/5] overflow-hidden rounded-xl border transition-[transform,box-shadow] duration-200 ease-out group-hover:-translate-y-1 group-active:translate-y-0 group-active:scale-[0.985] motion-reduce:transform-none"
									style:view-transition-name={projectViewTransitionName(p.id)}
									style:view-transition-class="morph-surface"
								>
									<!-- folded corner -->
									<div
										class="border-border bg-muted/60 absolute top-0 right-0 size-6 border-b border-l"
										style="clip-path: polygon(100% 0, 0 0, 100% 100%)"
									></div>
									<div class="flex h-full flex-col gap-2 p-5 pt-6">
										<div class="bg-foreground/80 h-2 w-3/5 rounded-full"></div>
										<div class="mt-2 flex flex-col gap-1.5">
											{#each lineWidths(p.id) as w, i (i)}
												<div class="bg-foreground/12 h-1.5 rounded-full" style:width={`${w}%`}></div>
											{/each}
										</div>
										<div class="mt-auto flex items-center gap-1.5">
											<span class="bg-brand/70 h-1.5 w-1.5 rounded-full"></span>
											<div class="bg-foreground/12 h-1.5 w-2/5 rounded-full"></div>
										</div>
									</div>
								</div>

								<!-- meta -->
								<div class="mt-2.5 px-0.5">
									{#if renaming === p.id}
										<!-- svelte-ignore a11y_autofocus -->
										<input
											bind:value={renameValue}
											class="bg-card border-border text-foreground focus-visible:border-ring w-full rounded-md border px-1.5 py-0.5 text-sm font-medium outline-none"
											autofocus
											onclick={(e) => e.stopPropagation()}
											onkeydown={(e) => {
												if (e.key === 'Enter') commitRename(p.id);
												if (e.key === 'Escape') renaming = null;
											}}
											onblur={() => commitRename(p.id)}
										/>
									{:else}
										<p class="text-foreground truncate text-sm font-medium">{p.name}</p>
									{/if}
									<p class="text-muted-foreground mt-0.5 truncate text-xs">
										{#if p.root}
											<span class="font-mono" title={p.root}>{p.root}</span>
										{:else}
											{p.files.length}
											{p.files.length === 1 ? 'file' : 'files'} · {relativeTime(p.updatedAt)}
										{/if}
									</p>
								</div>
							</button>

							<!-- per-card menu (native shadcn dropdown) -->
							<div class="absolute top-2 left-2">
								<DropdownMenu>
									<DropdownMenuTrigger>
										{#snippet child({ props })}
											<button
												{...props}
												class="bg-card/80 text-muted-foreground hover:text-foreground border-border grid size-7 place-items-center rounded-md border opacity-0 shadow-sm backdrop-blur transition-opacity group-hover:opacity-100 focus-visible:opacity-100 data-[state=open]:opacity-100"
												title="Project actions"
												aria-label="Project actions"
												onclick={(e) => e.stopPropagation()}
											>
												<IconDotsVertical size={15} />
											</button>
										{/snippet}
									</DropdownMenuTrigger>
									<DropdownMenuContent align="start" class="w-40">
										<DropdownMenuItem onclick={() => startRename(p)}>
											<IconPencil class="text-muted-foreground" /> Rename
										</DropdownMenuItem>
										<DropdownMenuItem onclick={() => onduplicate?.(p.id)}>
											<IconCopy class="text-muted-foreground" /> Duplicate
										</DropdownMenuItem>
										<DropdownMenuSeparator />
										<DropdownMenuItem variant="destructive" onclick={() => ondelete?.(p.id)}>
											<IconTrash /> Delete
										</DropdownMenuItem>
									</DropdownMenuContent>
								</DropdownMenu>
							</div>
						</div>
					{/each}
				</div>
			{/if}
		</div>
	</div>
</div>
