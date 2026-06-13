<script lang="ts" module>
	export type GitChange = { path: string; status: string; staged: boolean };
	export type GitCommitEntry = { hash: string; summary: string; author: string; time: number };
	export type GitHeadInfo = { branch: string | null; unborn: boolean };

	/** Host-injected Git backend (desktop = Tauri / gitoxide). */
	export type GitProvider = {
		isRepo: (root: string) => Promise<boolean>;
		init: (root: string) => Promise<void>;
		head: (root: string) => Promise<GitHeadInfo>;
		status: (root: string) => Promise<GitChange[]>;
		/** Stage paths into the index (`git add`). */
		stage: (root: string, paths: string[]) => Promise<void>;
		/** Unstage paths (`git reset <path>`). */
		unstage: (root: string, paths: string[]) => Promise<void>;
		/** Unified diff for a path; `staged` = HEAD↔index vs index↔worktree. */
		diff: (root: string, path: string, staged: boolean) => Promise<string>;
		/** Commit the staged index; returns the short hash. */
		commit: (root: string, message: string) => Promise<string>;
		log: (root: string, limit?: number) => Promise<GitCommitEntry[]>;
		/** Clone `url` into `dest`; returns the working-tree path. */
		clone: (url: string, dest: string) => Promise<string>;
		/** Configured remotes (name + fetch URL). */
		remotes: (root: string) => Promise<{ name: string; url: string }[]>;
		/** Fetch tracking refs (optionally from an authenticated `url`). */
		fetch: (root: string, url?: string) => Promise<void>;
		/** Fast-forward pull (system git); `url` carries token auth. */
		pull: (root: string, url?: string) => Promise<string>;
		/** Push current branch (system git); `url` carries token auth. */
		push: (root: string, url?: string, branch?: string) => Promise<string>;
	};
</script>

<script lang="ts">
	import { Button } from '@glyphx/ui/button';
	import { Textarea } from '@glyphx/ui/textarea';
	import {
		IconGitBranch,
		IconGitCommit,
		IconRefresh,
		IconHistory,
		IconPlus,
		IconMinus
	} from '@tabler/icons-svelte';

	/**
	 * Source Control — local version control for the open project folder. Stage /
	 * unstage working-tree changes against the real Git index, commit the staged
	 * set, and browse recent history. Remotes (push/pull) come later.
	 */
	let { git, root }: { git?: GitProvider; root?: string | null } = $props();

	let isRepo = $state(false);
	let head = $state<GitHeadInfo | undefined>(undefined);
	let changes = $state<GitChange[]>([]);
	let commits = $state<GitCommitEntry[]>([]);
	let message = $state('');

	let loading = $state(false);
	let busy = $state(false);
	let error = $state<string | undefined>(undefined);

	// Inline diff view for a clicked file.
	let openPath = $state<string | null>(null);
	let openStaged = $state(false);
	let diffText = $state('');
	let diffLoading = $state(false);

	async function showDiff(c: GitChange) {
		if (openPath === c.path && openStaged === c.staged) {
			openPath = null; // toggle closed
			return;
		}
		openPath = c.path;
		openStaged = c.staged;
		diffText = '';
		diffLoading = true;
		try {
			diffText = (await git?.diff(root!, c.path, c.staged)) ?? '';
		} catch (e) {
			diffText = String(e);
		} finally {
			diffLoading = false;
		}
	}

	const diffLines = $derived(diffText ? diffText.replace(/\n$/, '').split('\n') : []);
	function lineClass(l: string): string {
		if (l.startsWith('+') && !l.startsWith('+++')) return 'text-success';
		if (l.startsWith('-') && !l.startsWith('---')) return 'text-destructive';
		if (l.startsWith('@@')) return 'text-brand';
		return 'text-muted-foreground/80';
	}

	// Remotes
	let remotes = $state<{ name: string; url: string }[]>([]);
	let token = $state('');
	let showToken = $state(false);
	let remoteMsg = $state<string | undefined>(undefined);
	const originUrl = $derived(remotes.find((r) => r.name === 'origin')?.url);
	const hasRemote = $derived(!!originUrl);

	/** Origin URL with the access token injected for HTTPS auth (if entered). */
	function authedUrl(): string | undefined {
		if (!originUrl) return undefined;
		const t = token.trim();
		if (!t) return originUrl;
		try {
			const u = new URL(originUrl);
			u.username = 'x-access-token';
			u.password = t;
			return u.toString();
		} catch {
			return originUrl;
		}
	}

	const staged = $derived(changes.filter((c) => c.staged));
	const unstaged = $derived(changes.filter((c) => !c.staged));
	const canCommit = $derived(
		isRepo && !busy && message.trim().length > 0 && staged.length > 0
	);

	async function refresh() {
		if (!git || !root) return;
		loading = true;
		error = undefined;
		try {
			openPath = null; // close any stale inline diff
			isRepo = await git.isRepo(root);
			if (!isRepo) {
				changes = [];
				commits = [];
				head = undefined;
				return;
			}
			[head, changes, commits] = await Promise.all([
				git.head(root),
				git.status(root),
				git.log(root, 30)
			]);
			try {
				remotes = await git.remotes(root);
			} catch {
				remotes = [];
			}
		} catch (e) {
			error = String(e);
		} finally {
			loading = false;
		}
	}

	// Reload whenever the open folder changes.
	$effect(() => {
		void root;
		refresh();
	});

	async function run(fn: () => Promise<unknown>) {
		if (!git || !root || busy) return;
		busy = true;
		error = undefined;
		try {
			await fn();
			await refresh();
		} catch (e) {
			error = String(e);
		} finally {
			busy = false;
		}
	}

	const initRepo = () => run(() => git!.init(root!));
	const stage = (paths: string[]) => run(() => git!.stage(root!, paths));
	const unstage = (paths: string[]) => run(() => git!.unstage(root!, paths));

	function commit() {
		if (!canCommit) return;
		run(async () => {
			await git!.commit(root!, message.trim());
			message = '';
		});
	}

	const doFetch = () =>
		run(async () => {
			remoteMsg = undefined;
			await git!.fetch(root!, authedUrl());
			remoteMsg = 'Fetched.';
		});
	const doPull = () =>
		run(async () => {
			remoteMsg = undefined;
			remoteMsg = (await git!.pull(root!, authedUrl())) || 'Already up to date.';
		});
	const doPush = () =>
		run(async () => {
			remoteMsg = undefined;
			remoteMsg = (await git!.push(root!, authedUrl(), head?.branch ?? undefined)) || 'Pushed.';
		});

	const STATUS_LABEL: Record<string, string> = {
		modified: 'M',
		deleted: 'D',
		untracked: 'U',
		added: 'A',
		renamed: 'R'
	};
	const STATUS_CLASS: Record<string, string> = {
		modified: 'text-warning',
		deleted: 'text-destructive',
		untracked: 'text-success',
		added: 'text-success',
		renamed: 'text-brand'
	};

	function when(secs: number): string {
		return new Date(secs * 1000).toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
	}
</script>

{#snippet row(c: GitChange, action: 'stage' | 'unstage')}
	{@const open = openPath === c.path && openStaged === c.staged}
	<div>
		<div class="hover:bg-muted/60 group flex items-center gap-1.5 rounded px-1 py-0.5 text-xs">
			<button
				class="text-foreground/90 hover:text-foreground min-w-0 flex-1 truncate text-left {open
					? 'font-medium'
					: ''}"
				title="Show diff — {c.path}"
				onclick={() => showDiff(c)}
			>
				{c.path}
			</button>
			<Button
				variant="ghost"
				size="icon-xs"
				class="opacity-0 group-hover:opacity-100"
				title={action === 'stage' ? 'Stage' : 'Unstage'}
				aria-label={action === 'stage' ? 'Stage' : 'Unstage'}
				disabled={busy}
				onclick={() => (action === 'stage' ? stage([c.path]) : unstage([c.path]))}
			>
				{#if action === 'stage'}<IconPlus size={13} />{:else}<IconMinus size={13} />{/if}
			</Button>
			<span class="shrink-0 font-mono text-[10px] {STATUS_CLASS[c.status] ?? ''}">
				{STATUS_LABEL[c.status] ?? '?'}
			</span>
		</div>
		{#if open}
			<div class="border-border/60 mx-1 my-1 overflow-auto rounded border">
				{#if diffLoading}
					<p class="text-muted-foreground p-2 text-[11px]">Loading diff…</p>
				{:else if !diffLines.length}
					<p class="text-muted-foreground p-2 text-[11px]">No textual diff (binary or empty).</p>
				{:else}
					<pre class="max-h-64 overflow-auto p-2 font-mono text-[10.5px] leading-snug"><!--
					-->{#each diffLines as l, i (i)}<span class="{lineClass(l)} block whitespace-pre">{l || ' '}</span>{/each}</pre>
				{/if}
			</div>
		{/if}
	</div>
{/snippet}

{#if !root}
	<div class="text-muted-foreground flex flex-col items-center gap-2 px-2 py-8 text-center text-xs">
		<IconGitBranch size={22} />
		<p>Open a folder to use source control.</p>
	</div>
{:else if !isRepo}
	<div class="text-muted-foreground flex flex-col items-center gap-2 px-2 py-8 text-center text-xs">
		<IconGitBranch size={22} />
		<p>This folder isn't a Git repository yet.</p>
		<Button variant="outline" size="sm" class="mt-1" disabled={busy} onclick={initRepo}>
			{busy ? 'Initializing…' : 'Initialize repository'}
		</Button>
	</div>
{:else}
	<div class="flex flex-col gap-2 px-1">
		<!-- Branch + refresh -->
		<div class="flex items-center gap-1.5 px-0.5">
			<IconGitBranch size={13} class="text-muted-foreground shrink-0" />
			<span class="text-foreground truncate text-xs font-medium">
				{head?.branch ?? 'HEAD'}{head?.unborn ? ' (no commits yet)' : ''}
			</span>
			<Button
				variant="ghost"
				size="icon-xs"
				class="ml-auto"
				title="Refresh"
				aria-label="Refresh"
				disabled={loading}
				onclick={refresh}
			>
				<IconRefresh size={13} />
			</Button>
		</div>

		<!-- Commit box -->
		<Textarea bind:value={message} placeholder="Commit message" rows={2} class="resize-none text-xs" />
		<Button size="sm" disabled={!canCommit} onclick={commit}>
			<IconGitCommit size={14} />
			{busy ? 'Committing…' : `Commit${staged.length ? ` ${staged.length}` : ''}`}
		</Button>

		{#if error}
			<p class="text-destructive px-0.5 text-[11px] leading-snug">{error}</p>
		{/if}

		<!-- Staged -->
		{#if staged.length}
			<div class="mt-1 flex flex-col gap-0.5">
				<div class="flex items-center px-0.5">
					<span class="text-muted-foreground text-[10px] font-medium tracking-wide uppercase">
						Staged Changes ({staged.length})
					</span>
					<Button
						variant="ghost"
						size="xs"
						class="ml-auto"
						disabled={busy}
						onclick={() => unstage(staged.map((c) => c.path))}
					>
						Unstage all
					</Button>
				</div>
				{#each staged as c (c.path)}
					{@render row(c, 'unstage')}
				{/each}
			</div>
		{/if}

		<!-- Changes (unstaged) -->
		<div class="mt-1 flex flex-col gap-0.5">
			<div class="flex items-center px-0.5">
				<span class="text-muted-foreground text-[10px] font-medium tracking-wide uppercase">
					Changes{unstaged.length ? ` (${unstaged.length})` : ''}
				</span>
				{#if unstaged.length}
					<Button
						variant="ghost"
						size="xs"
						class="ml-auto"
						disabled={busy}
						onclick={() => stage(unstaged.map((c) => c.path))}
					>
						Stage all
					</Button>
				{/if}
			</div>
			{#if !unstaged.length}
				<p class="text-muted-foreground/70 px-0.5 py-1 text-[11px]">
					{loading ? 'Checking…' : staged.length ? 'Nothing else to stage.' : 'No changes.'}
				</p>
			{:else}
				{#each unstaged as c (c.path)}
					{@render row(c, 'stage')}
				{/each}
			{/if}
		</div>

		<!-- Remote -->
		{#if hasRemote}
			<div class="border-border/60 mt-1 flex flex-col gap-1 border-t pt-1.5">
				<div class="flex items-center gap-1 px-0.5">
					<span class="text-muted-foreground text-[10px] font-medium tracking-wide uppercase">
						Remote
					</span>
					<button
						class="text-muted-foreground hover:text-foreground ml-auto text-[10px]"
						onclick={() => (showToken = !showToken)}
					>
						{showToken ? 'Hide token' : 'Token'}
					</button>
				</div>
				{#if showToken}
					<input
						type="password"
						bind:value={token}
						placeholder="Access token (for private repos)"
						class="border-border bg-background focus:ring-brand/40 rounded border px-2 py-1 text-xs outline-none focus:ring-2"
					/>
				{/if}
				<div class="flex gap-1.5">
					<Button variant="outline" size="xs" disabled={busy} onclick={doFetch}>Fetch</Button>
					<Button variant="outline" size="xs" disabled={busy} onclick={doPull}>Pull</Button>
					<Button variant="outline" size="xs" disabled={busy} onclick={doPush}>Push</Button>
				</div>
				{#if remoteMsg}
					<p class="text-muted-foreground/80 px-0.5 text-[11px] leading-snug break-words">
						{remoteMsg}
					</p>
				{/if}
			</div>
		{/if}

		<!-- History -->
		{#if commits.length}
			<div class="border-border/60 mt-1 flex flex-col gap-0.5 border-t pt-1.5">
				<span
					class="text-muted-foreground flex items-center gap-1 px-0.5 text-[10px] font-medium tracking-wide uppercase"
				>
					<IconHistory size={11} /> History
				</span>
				{#each commits as c (c.hash)}
					<div class="flex items-baseline gap-1.5 px-0.5 py-0.5 text-xs" title={c.author}>
						<span class="text-muted-foreground/70 shrink-0 font-mono text-[10px]">{c.hash}</span>
						<span class="text-foreground/85 truncate">{c.summary}</span>
						<span class="text-muted-foreground/50 ml-auto shrink-0 text-[10px]">{when(c.time)}</span>
					</div>
				{/each}
			</div>
		{/if}
	</div>
{/if}
