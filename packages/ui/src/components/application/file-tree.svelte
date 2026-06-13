<script lang="ts" module>
	export type TreeFile = { type: 'file'; id: string; name: string };
	export type TreeFolder = { type: 'folder'; name: string; path: string; children: TreeNode[] };
	export type TreeNode = TreeFile | TreeFolder;
</script>

<script lang="ts">
	import Self from './file-tree.svelte';
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuTrigger
	} from '@glyph/ui/dropdown-menu';
	import { slide } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import {
		IconFileText,
		IconFolder,
		IconFolderOpen,
		IconChevronRight,
		IconDots,
		IconPencil,
		IconTrash,
		IconTargetArrow
	} from '@tabler/icons-svelte';

	/**
	 * FileTree — recursive Explorer tree. Folders expand/collapse with a smooth
	 * `slide` (the chevron rotates on the same curve); files are selectable and
	 * carry hover actions (set-as-main / rename inline / delete). The compile
	 * target (`mainId`) shows a "main" badge. Open-state is local per folder
	 * (default open). Indentation is depth-driven.
	 */
	let {
		nodes,
		activeId = '',
		mainId = null,
		depth = 0,
		open = $bindable({}),
		onopen,
		onrename,
		ondelete,
		onsetmain
	}: {
		nodes: TreeNode[];
		activeId?: string;
		/** Id of the project's main (compile-target) file. */
		mainId?: string | null;
		depth?: number;
		open?: Record<string, boolean>;
		onopen?: (id: string) => void;
		/** Rename a file — receives the new leaf name (folder prefix is preserved upstream). */
		onrename?: (id: string, name: string) => void;
		ondelete?: (id: string) => void;
		/** Mark a file as the compile target. Omitted when there's no project. */
		onsetmain?: (id: string) => void;
	} = $props();

	const isTex = (name: string) => /\.tex$/i.test(name);

	const isOpen = (path: string) => open[path] ?? true;
	const toggle = (path: string) => (open[path] = !isOpen(path));
	const indent = (d: number) => `${d * 12 + 8}px`;

	let renamingId = $state<string | null>(null);
	let renameValue = $state('');

	function startRename(node: TreeFile) {
		renamingId = node.id;
		renameValue = node.name;
	}
	function commitRename(id: string) {
		const name = renameValue.trim();
		if (name) onrename?.(id, name);
		renamingId = null;
	}
</script>

{#each nodes as node (node.type === 'folder' ? `d:${node.path}` : `f:${node.id}`)}
	{#if node.type === 'folder'}
		<button
			class="text-muted-foreground hover:bg-muted hover:text-foreground flex w-full items-center gap-1 rounded py-1 pr-2 text-left transition-colors"
			style:padding-left={indent(depth)}
			aria-expanded={isOpen(node.path)}
			onclick={() => toggle(node.path)}
		>
			<IconChevronRight
				size={13}
				class="shrink-0 transition-transform duration-200 ease-[cubic-bezier(0.25,1,0.5,1)] {isOpen(
					node.path
				)
					? 'rotate-90'
					: ''}"
			/>
			{#if isOpen(node.path)}
				<IconFolderOpen size={15} class="text-muted-foreground shrink-0" />
			{:else}
				<IconFolder size={15} class="text-muted-foreground shrink-0" />
			{/if}
			<span class="truncate">{node.name}</span>
		</button>
		{#if isOpen(node.path)}
			<div transition:slide={{ duration: 200, easing: cubicOut }}>
				<Self
					nodes={node.children}
					{activeId}
					{mainId}
					depth={depth + 1}
					bind:open
					{onopen}
					{onrename}
					{ondelete}
					{onsetmain}
				/>
			</div>
		{/if}
	{:else}
		<div class="group/row relative flex items-center">
			{#if renamingId === node.id}
				<div class="flex w-full items-center gap-1 py-1 pr-2" style:padding-left={indent(depth)}>
					<span class="w-[13px] shrink-0"></span>
					<IconFileText size={15} class="text-muted-foreground shrink-0" />
					<!-- svelte-ignore a11y_autofocus -->
					<input
						bind:value={renameValue}
						class="bg-background border-ring text-foreground min-w-0 flex-1 rounded border px-1 py-0 text-[13px] outline-none"
						autofocus
						spellcheck="false"
						onclick={(e) => e.stopPropagation()}
						onkeydown={(e) => {
							if (e.key === 'Enter') commitRename(node.id);
							if (e.key === 'Escape') renamingId = null;
						}}
						onblur={() => commitRename(node.id)}
					/>
				</div>
			{:else}
				<button
					class="flex w-full items-center gap-1 rounded py-1 pr-7 text-left transition-colors {node.id ===
					activeId
						? 'bg-accent text-accent-foreground'
						: 'text-muted-foreground hover:bg-muted hover:text-foreground'}"
					style:padding-left={indent(depth)}
					aria-current={node.id === activeId}
					title={node.name}
					onclick={() => onopen?.(node.id)}
					ondblclick={() => startRename(node)}
				>
					<span class="w-[13px] shrink-0"></span>
					<IconFileText size={15} class="shrink-0 {node.id === activeId ? 'text-brand' : ''}" />
					<span class="truncate">{node.name}</span>
					{#if node.id === mainId}
						<span
							class="bg-brand-subtle text-brand ml-1 shrink-0 rounded px-1 text-[9px] font-semibold tracking-wide uppercase"
							title="Main file (compile target)"
						>
							main
						</span>
					{/if}
				</button>

				<DropdownMenu>
					<DropdownMenuTrigger>
						{#snippet child({ props })}
							<button
								{...props}
								class="text-muted-foreground hover:bg-muted hover:text-foreground absolute top-1/2 right-1 grid size-5 -translate-y-1/2 place-items-center rounded opacity-0 transition-opacity group-hover/row:opacity-100 focus-visible:opacity-100 data-[state=open]:opacity-100"
								title="File actions"
								aria-label="File actions"
								onclick={(e: MouseEvent) => e.stopPropagation()}
							>
								<IconDots size={14} />
							</button>
						{/snippet}
					</DropdownMenuTrigger>
					<DropdownMenuContent align="end" class="w-40">
						{#if onsetmain && isTex(node.name) && node.id !== mainId}
							<DropdownMenuItem onSelect={() => onsetmain?.(node.id)}>
								<IconTargetArrow class="text-muted-foreground" /> Set as main
							</DropdownMenuItem>
						{/if}
						<DropdownMenuItem onSelect={() => startRename(node)}>
							<IconPencil class="text-muted-foreground" /> Rename
						</DropdownMenuItem>
						<DropdownMenuItem variant="destructive" onSelect={() => ondelete?.(node.id)}>
							<IconTrash /> Delete
						</DropdownMenuItem>
					</DropdownMenuContent>
				</DropdownMenu>
			{/if}
		</div>
	{/if}
{/each}
