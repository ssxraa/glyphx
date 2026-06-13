<script lang="ts" module>
	export type TreeFile = { type: 'file'; id: string; name: string };
	export type TreeFolder = { type: 'folder'; name: string; path: string; children: TreeNode[] };
	export type TreeNode = TreeFile | TreeFolder;
</script>

<script lang="ts">
	import Self from './file-tree.svelte';
	import { slide } from 'svelte/transition';
	import { cubicOut } from 'svelte/easing';
	import { IconFileText, IconFolder, IconFolderOpen, IconChevronRight } from '@tabler/icons-svelte';

	/**
	 * FileTree — recursive Explorer tree. Folders expand/collapse with a smooth
	 * `slide` (the chevron rotates on the same curve); files are selectable.
	 * Open-state is local per folder (default open). Indentation is depth-driven.
	 */
	let {
		nodes,
		activeId = '',
		depth = 0,
		open = $bindable({}),
		onopen
	}: {
		nodes: TreeNode[];
		activeId?: string;
		depth?: number;
		open?: Record<string, boolean>;
		onopen?: (id: string) => void;
	} = $props();

	const isOpen = (path: string) => open[path] ?? true;
	const toggle = (path: string) => (open[path] = !isOpen(path));
	const indent = (d: number) => `${d * 12 + 8}px`;
</script>

{#each nodes as node (node.type === 'folder' ? `d:${node.path}` : `f:${node.id}`)}
	{#if node.type === 'folder'}
		<button
			class="text-muted-foreground hover:bg-muted hover:text-foreground flex w-full items-center gap-1.5 rounded-md py-1.5 pr-2 text-left transition-colors"
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
				<Self nodes={node.children} {activeId} depth={depth + 1} bind:open {onopen} />
			</div>
		{/if}
	{:else}
		<button
			class="flex w-full items-center gap-1.5 rounded-md py-1.5 pr-2 text-left transition-colors {node.id ===
			activeId
				? 'bg-accent text-accent-foreground'
				: 'text-muted-foreground hover:bg-muted hover:text-foreground'}"
			style:padding-left={indent(depth)}
			aria-current={node.id === activeId}
			onclick={() => onopen?.(node.id)}
		>
			<span class="w-[13px] shrink-0"></span>
			<IconFileText size={15} class="shrink-0 {node.id === activeId ? 'text-brand' : ''}" />
			<span class="truncate">{node.name}</span>
		</button>
	{/if}
{/each}
