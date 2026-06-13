<script lang="ts" module>
	export type PaletteFile = { id: string; name: string };
</script>

<script lang="ts">
	import {
		CommandDialog,
		CommandEmpty,
		CommandGroup,
		CommandInput,
		CommandItem,
		CommandList
	} from '@glyphx/ui/command';
	import { IconChevronDown, IconCornerDownLeft, IconFile, IconStack2 } from '@tabler/icons-svelte';

	/**
	 * CommandPalette — the centre of the top bar. Shows the workspace name as a
	 * VS Code-style pill; clicking it (or ⌘/Ctrl+P) opens the native command
	 * dialog (cmdk: built-in filtering, keyboard nav, focus trap, portal) to jump
	 * between files.
	 */
	let {
		open = $bindable(false),
		files = [],
		activeId = '',
		projectName = 'Project',
		onopen
	}: {
		open?: boolean;
		files?: PaletteFile[];
		activeId?: string;
		projectName?: string;
		onopen?: (id: string) => void;
	} = $props();

	function choose(id: string) {
		onopen?.(id);
		open = false;
	}

	// Split a path into folder / file so each row reads like an explorer entry.
	function parts(name: string) {
		const i = name.lastIndexOf('/');
		return i === -1
			? { dir: '', base: name }
			: { dir: name.slice(0, i + 1), base: name.slice(i + 1) };
	}
</script>

<!-- Centre trigger: the workspace name (VS Code's command-centre slot). -->
<button
	class="text-muted-foreground hover:bg-muted/60 hover:text-foreground border-border/60 bg-muted/30 flex h-7 w-72 max-w-[40vw] items-center justify-center gap-2 rounded-md border px-3 text-[13px] transition-colors"
	title="Search files (⌘/Ctrl+P)"
	aria-haspopup="dialog"
	onclick={() => (open = true)}
>
	<IconStack2 size={14} class="shrink-0 opacity-70" />
	<span class="text-foreground truncate font-medium">{projectName}</span>
	<IconChevronDown size={13} class="shrink-0 opacity-50" />
</button>

<CommandDialog
	bind:open
	title="Go to file"
	description="Search files by name"
	class="sm:max-w-[34rem]"
>
	<CommandInput placeholder="Go to file by name…" />
	<CommandList>
		<CommandEmpty>No matching files</CommandEmpty>
		<CommandGroup heading={projectName}>
			{#each files as f (f.id)}
				{@const p = parts(f.name)}
				<CommandItem value={f.name} onSelect={() => choose(f.id)} class="gap-2.5 py-2">
					<IconFile class="text-muted-foreground shrink-0" />
					<span class="text-foreground truncate">{p.base}</span>
					{#if p.dir}
						<span class="text-muted-foreground/60 truncate text-xs">{p.dir}</span>
					{/if}
					<!-- data-slot=command-shortcut suppresses the default check indicator
					     and keeps this group flush-right. -->
					<span data-slot="command-shortcut" class="ml-auto flex shrink-0 items-center gap-2">
						{#if f.id === activeId}
							<span class="text-muted-foreground/50 text-[11px]">open</span>
						{/if}
						<IconCornerDownLeft
							size={13}
							class="text-muted-foreground/40 opacity-0 transition-opacity group-data-[selected=true]/command-item:opacity-100"
						/>
					</span>
				</CommandItem>
			{/each}
		</CommandGroup>
	</CommandList>
</CommandDialog>
