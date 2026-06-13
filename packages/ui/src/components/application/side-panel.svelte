<script lang="ts">
	import { Button } from '@glyph/ui/button';
	import { Segmented } from '@glyph/ui/segmented';
	import { Separator } from '@glyph/ui/separator';
	import { SettingsField } from '@glyph/ui/settings-field';
	import {
	  EDITOR_FONT_LABELS,
	  settings,
	  type Appearance,
	  type EditorFont,
	  type LatexGrammar
	} from '@glyph/ui/settings';
	import { Switch } from '@glyph/ui/switch';
	import {
	  IconChevronRight,
	  IconChevronUp,
	  IconChevronDown,
	  IconFolderOpen,
	  IconGitBranch,
	  IconMinus,
	  IconPlus,
	  IconReplace,
	  IconReplaceFilled
	} from '@tabler/icons-svelte';
	import { cubicOut } from 'svelte/easing';
	import { slide } from 'svelte/transition';
	import type { ActivityView } from './activity-bar.svelte';
	import EngineSettings, { type EngineManager } from './engine-settings.svelte';
	import FileTree, { type TreeNode } from './file-tree.svelte';

	type FileMeta = { id: string; name: string };
	type SearchOptions = {
		query: string;
		replace?: string;
		caseSensitive?: boolean;
		wholeWord?: boolean;
		regexp?: boolean;
	};
	type SearchMatch = { from: number; to: number; line: number; column: number; text: string };

	/**
	 * SidePanel — content for the active rail view. Explorer switches files;
	 * Settings edits the live preferences (Segmented choices + Mac-style
	 * switches). Search is a full find/replace panel wired to the editor. Git is
	 * a placeholder.
	 */
	let {
		view = 'files',
		files = [],
		activeId = '',
		mainId = null,
		projectName = 'Project',
		hasProject = false,
		widthPx = 240,
		engine,
		onopen,
		onnew,
		onopenfolder,
		onrenamefile,
		ondeletefile,
		onsetmain,
		onregistershell,
		searchResults = [],
		searchActive = 0,
		onsearch,
		ongotoresult,
		onsearchnext,
		onsearchprev,
		onreplacecurrent,
		onreplaceall
	}: {
		view?: ActivityView;
		files?: FileMeta[];
		activeId?: string;
		/** Absolute path / id of the project's main (compile-target) file. */
		mainId?: string | null;
		projectName?: string;
		/** Whether a folder-based project host is available (enables Open Folder). */
		hasProject?: boolean;
		widthPx?: number;
		engine?: EngineManager;
		onopen?: (id: string) => void;
		onnew?: () => void;
		onopenfolder?: () => void;
		onrenamefile?: (id: string, name: string) => void;
		ondeletefile?: (id: string) => void;
		onsetmain?: (id: string) => void;
		/** Register the OS "Open with Glyph" folder integration (desktop). */
		onregistershell?: () => void;
		searchResults?: SearchMatch[];
		searchActive?: number;
		onsearch?: (o: SearchOptions) => void;
		ongotoresult?: (i: number) => void;
		onsearchnext?: () => void;
		onsearchprev?: () => void;
		onreplacecurrent?: (replace: string) => void;
		onreplaceall?: (replace: string) => void;
	} = $props();

	const heading = $derived(
		view === 'files'
			? 'Explorer'
			: view === 'search'
				? 'Search'
				: view === 'git'
					? 'Source Control'
					: 'Settings'
	);

	// Folder-based project tree — file names split on "/" nest into folders, so
	// the Explorer is folder-ready (flat files today, real folders later).
	function buildTree(items: FileMeta[]): TreeNode[] {
		const root: TreeNode[] = [];
		const folders = new Map<string, TreeNode[]>();
		for (const f of items) {
			const parts = f.name.split('/');
			let level = root;
			let path = '';
			for (let i = 0; i < parts.length - 1; i++) {
				path = path ? `${path}/${parts[i]}` : parts[i];
				let children = folders.get(path);
				if (!children) {
					children = [];
					folders.set(path, children);
					level.push({ type: 'folder', name: parts[i], path, children });
				}
				level = children;
			}
			level.push({ type: 'file', id: f.id, name: parts[parts.length - 1] });
		}
		return root;
	}
	// Root files live at the root (VS Code style); only real subfolders nest.
	const rootNodes = $derived<TreeNode[]>(buildTree(files));
	let treeOpen = $state<Record<string, boolean>>({});
	let rootExpanded = $state(true);

	const appearanceOpts: { value: Appearance; label: string }[] = [
		{ value: 'light', label: 'Light' },
		{ value: 'dark', label: 'Dark' },
		{ value: 'system', label: 'System' }
	];
	const grammarOpts: { value: LatexGrammar; label: string }[] = [
		{ value: 'legacy', label: 'stex' },
		{ value: 'lezer', label: 'lezer' }
	];
	const fontOpts = (Object.keys(EDITOR_FONT_LABELS) as EditorFont[]).map((id) => ({
		value: id,
		label: EDITOR_FONT_LABELS[id]
	}));

	// --- Find / replace (wired to the editor via callbacks) ---
	let query = $state('');
	let replace = $state('');
	let matchCase = $state(false);
	let wholeWord = $state(false);
	let useRegex = $state(false);
	let showReplace = $state(false);
	let searchInputEl = $state<HTMLInputElement>();

	function emitSearch() {
		onsearch?.({
			query,
			replace,
			caseSensitive: matchCase,
			wholeWord,
			regexp: useRegex
		});
	}
	const findOptions = $derived([
		{
			key: 'case',
			label: 'Aa',
			title: 'Match case',
			on: matchCase,
			toggle: () => {
				matchCase = !matchCase;
				emitSearch();
			}
		},
		{
			key: 'word',
			label: 'W',
			title: 'Whole word',
			on: wholeWord,
			toggle: () => {
				wholeWord = !wholeWord;
				emitSearch();
			}
		},
		{
			key: 'regex',
			label: '.*',
			title: 'Regular expression',
			on: useRegex,
			toggle: () => {
				useRegex = !useRegex;
				emitSearch();
			}
		}
	]);

	function onSearchKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			if (e.shiftKey) onsearchprev?.();
			else onsearchnext?.();
		}
	}

	// Autofocus the field when the Search view opens (e.g. via Ctrl/Cmd+F).
	$effect(() => {
		if (view === 'search') searchInputEl?.focus();
	});
</script>

<aside
	class="bg-card border-border flex shrink-0 flex-col border-r min-h-dvh"
	style:width={`${widthPx}px`}
	aria-label={heading}
>
	<div
		class="text-muted-foreground flex h-9 shrink-0 items-center justify-between px-3 text-xs font-medium tracking-wider uppercase"
	>
		<span>{heading}</span>
		{#if view === 'files'}
			<div class="-mr-1 flex items-center gap-0.5">
				<button
					class="hover:bg-muted hover:text-foreground grid size-6 place-items-center rounded transition-colors"
					title="New file"
					aria-label="New file"
					onclick={() => onnew?.()}
				>
					<IconPlus size={15} />
				</button>
				{#if hasProject}
					<button
						class="hover:bg-muted hover:text-foreground grid size-6 place-items-center rounded transition-colors"
						title="Open folder (⌘/Ctrl+O)"
						aria-label="Open folder"
						onclick={() => onopenfolder?.()}
					>
						<IconFolderOpen size={15} />
					</button>
				{/if}
			</div>
		{/if}
	</div>

	<div class="min-h-0 flex-1 overflow-auto px-1.5 pb-2 text-[13px]">
		{#if view === 'files'}
			<!-- Workspace root header (the project / directory name, VS Code style). -->
			<button
				class="text-foreground hover:bg-muted/60 flex w-full items-center gap-1 rounded-md px-1.5 py-1 text-xs font-semibold tracking-wide uppercase transition-colors"
				aria-expanded={rootExpanded}
				onclick={() => (rootExpanded = !rootExpanded)}
			>
				<IconChevronRight
					size={13}
					class="shrink-0 transition-transform duration-200 ease-[cubic-bezier(0.25,1,0.5,1)] {rootExpanded
						? 'rotate-90'
						: ''}"
				/>
				<span class="truncate">{projectName}</span>
			</button>
			{#if rootExpanded}
				<div transition:slide={{ duration: 200, easing: cubicOut }}>
					<FileTree
						nodes={rootNodes}
						{activeId}
						{mainId}
						bind:open={treeOpen}
						onopen={(id) => onopen?.(id)}
						onrename={(id, name) => onrenamefile?.(id, name)}
						ondelete={(id) => ondeletefile?.(id)}
						onsetmain={hasProject ? (id) => onsetmain?.(id) : undefined}
					/>
				</div>
			{/if}
		{:else if view === 'search'}
			<div class="flex flex-col gap-1 pt-0.5">
				<div class="flex items-start gap-0.5">
					<button
						class="text-muted-foreground hover:bg-muted hover:text-foreground mt-0.5 grid size-5 shrink-0 place-items-center rounded transition-colors"
						title={showReplace ? 'Hide replace' : 'Toggle replace'}
						aria-label="Toggle replace"
						aria-expanded={showReplace}
						onclick={() => (showReplace = !showReplace)}
					>
						<IconChevronRight
							size={14}
							class="transition-transform duration-200 {showReplace ? 'rotate-90' : ''}"
						/>
					</button>

					<div class="flex min-w-0 flex-1 flex-col gap-1">
						<!-- Find — toggles live inside the field (VS Code parity) -->
						<div class="relative">
							<input
								bind:this={searchInputEl}
								bind:value={query}
								oninput={emitSearch}
								onkeydown={onSearchKeydown}
								class="bg-background border-border text-foreground placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/40 h-7 w-full rounded border py-1 pr-16 pl-2 text-[13px] outline-none transition-[box-shadow,border-color] focus-visible:ring-1"
								placeholder="Find"
								aria-label="Find in document"
								spellcheck="false"
							/>
							<div class="absolute top-1/2 right-1 flex -translate-y-1/2 items-center gap-0.5">
								{#each findOptions as opt (opt.key)}
									<button
										class="grid size-[18px] place-items-center rounded font-mono text-[10px] leading-none transition-colors {opt.on
											? 'bg-brand-subtle text-brand'
											: 'text-muted-foreground hover:bg-muted hover:text-foreground'}"
										title={opt.title}
										aria-label={opt.title}
										aria-pressed={opt.on}
										onclick={opt.toggle}
									>
										{opt.label}
									</button>
								{/each}
							</div>
						</div>

						<!-- Match count + nav -->
						<div class="flex items-center gap-1">
							<span class="text-muted-foreground/70 text-xs tabular-nums">
								{#if query && searchResults.length}
									{searchActive + 1} of {searchResults.length}
								{:else if query}
									No results
								{/if}
							</span>
							<button
								class="text-muted-foreground hover:bg-muted hover:text-foreground ml-auto grid size-5 place-items-center rounded transition-colors disabled:opacity-40"
								title="Previous match (Shift+Enter)"
								aria-label="Previous match"
								disabled={!searchResults.length}
								onclick={() => onsearchprev?.()}
							>
								<IconChevronUp size={15} />
							</button>
							<button
								class="text-muted-foreground hover:bg-muted hover:text-foreground grid size-5 place-items-center rounded transition-colors disabled:opacity-40"
								title="Next match (Enter)"
								aria-label="Next match"
								disabled={!searchResults.length}
								onclick={() => onsearchnext?.()}
							>
								<IconChevronDown size={15} />
							</button>
						</div>

						<!-- Replace — "replace all" button lives inside the field -->
						{#if showReplace}
							<div class="relative">
								<input
									bind:value={replace}
									class="bg-background border-border text-foreground placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/40 h-7 w-full rounded border py-1 pr-12 pl-2 text-[13px] outline-none transition-[box-shadow,border-color] focus-visible:ring-1"
									placeholder={useRegex ? 'Replace ($1, $&…)' : 'Replace'}
									aria-label="Replace with"
									spellcheck="false"
								/>
								<div class="absolute top-1/2 right-1 flex -translate-y-1/2 items-center gap-0.5">
									<button
										class="text-muted-foreground hover:bg-muted hover:text-foreground grid size-[18px] place-items-center rounded transition-colors disabled:opacity-40"
										title="Replace next match"
										aria-label="Replace next match"
										disabled={!searchResults.length}
										onclick={() => onreplacecurrent?.(replace)}
									>
										<IconReplace size={14} />
									</button>
									<button
										class="text-muted-foreground hover:bg-muted hover:text-foreground grid size-[18px] place-items-center rounded transition-colors disabled:opacity-40"
										title="Replace all matches"
										aria-label="Replace all matches"
										disabled={!searchResults.length}
										onclick={() => onreplaceall?.(replace)}
									>
										<IconReplaceFilled size={14} />
									</button>
								</div>
							</div>
						{/if}
					</div>
				</div>

				<!-- Results list -->
				{#if query && searchResults.length}
					<ul class="mt-1 flex flex-col">
						{#each searchResults.slice(0, 500) as m, i (i)}
							<li>
								<button
									class="flex w-full items-baseline gap-1.5 rounded px-2 py-0.5 text-left transition-colors {i ===
									searchActive
										? 'bg-accent text-accent-foreground'
										: 'hover:bg-muted text-muted-foreground'}"
									onclick={() => ongotoresult?.(i)}
									title={`Line ${m.line}`}
								>
									<span class="text-muted-foreground/50 w-7 shrink-0 text-right font-mono text-[10px] tabular-nums">
										{m.line}
									</span>
									<span class="truncate font-mono text-[11px]">{m.text.trim() || ' '}</span>
								</button>
							</li>
						{/each}
					</ul>
					{#if searchResults.length > 500}
						<p class="text-muted-foreground/60 px-2 text-[11px]">
							Showing first 500 of {searchResults.length}.
						</p>
					{/if}
				{:else if !query}
					<p class="text-muted-foreground/70 mt-1 px-1.5 text-[11px]">
						Find &amp; replace in the active file.
					</p>
				{/if}
			</div>
		{:else if view === 'git'}
			<div
				class="text-muted-foreground flex flex-col items-center gap-2 px-2 py-8 text-center text-xs"
			>
				<IconGitBranch size={22} />
				<p>No source control provider connected.</p>
				<Button variant="outline" size="sm" class="mt-1">Initialize repository</Button>
			</div>
		{:else}
			<!-- Settings — same SettingsField / Separator primitives as the
			     /settings route, in the compact (sm) size. -->
			<div class="flex flex-col gap-4 px-1 pt-0.5 pb-2">
				<SettingsField size="sm" label="Appearance">
					<Segmented
						options={appearanceOpts}
						value={settings.appearance}
						onValueChange={(v) => (settings.appearance = v)}
						size="sm"
						aria-label="Appearance"
					/>
				</SettingsField>

				<SettingsField size="sm" label="LaTeX grammar">
					<Segmented
						options={grammarOpts}
						value={settings.grammar}
						onValueChange={(v) => (settings.grammar = v)}
						size="sm"
						aria-label="LaTeX grammar"
					/>
				</SettingsField>

				<SettingsField size="sm" label="Editor font">
					<Segmented
						options={fontOpts}
						value={settings.font}
						onValueChange={(v) => (settings.font = v)}
						size="sm"
						aria-label="Editor font"
					/>
				</SettingsField>

				<SettingsField size="sm" label="Editor font size" layout="row">
					<div class="flex items-center gap-1">
						<Button
							variant="outline"
							size="icon-sm"
							aria-label="Decrease font size"
							disabled={settings.fontSize <= 10}
							onclick={() => (settings.fontSize = Math.max(10, settings.fontSize - 1))}
						>
							<IconMinus size={15} />
						</Button>
						<span class="text-foreground w-9 text-center text-[13px] tabular-nums">
							{settings.fontSize}px
						</span>
						<Button
							variant="outline"
							size="icon-sm"
							aria-label="Increase font size"
							disabled={settings.fontSize >= 24}
							onclick={() => (settings.fontSize = Math.min(24, settings.fontSize + 1))}
						>
							<IconPlus size={15} />
						</Button>
					</div>
				</SettingsField>

				<Separator />

				<SettingsField size="sm" label="Line wrapping" layout="row">
					<Switch
						checked={settings.lineWrapping}
						onCheckedChange={(v) => (settings.lineWrapping = v)}
						aria-label="Line wrapping"
					/>
				</SettingsField>

				<SettingsField
					size="sm"
					label="Live compile"
					description="Recompile as you type"
					layout="row"
				>
					<Switch
						checked={settings.autoCompile}
						onCheckedChange={(v) => (settings.autoCompile = v)}
						aria-label="Live compile"
					/>
				</SettingsField>

				{#if engine}
					<EngineSettings {engine} />
				{/if}

				{#if onregistershell}
					<Separator />
					<SettingsField
						size="sm"
						label="System integration"
						description="Add an “Open with Glyph” entry to the folder right-click menu. (.tex and .glyx files are associated by the installer.)"
					>
						<Button
							variant="outline"
							size="xs"
							class="self-start"
							onclick={() => onregistershell?.()}
						>
							Add “Open with Glyph”
						</Button>
					</SettingsField>
				{/if}
			</div>
		{/if}
	</div>
</aside>
