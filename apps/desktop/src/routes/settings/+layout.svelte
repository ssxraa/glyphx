<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page } from '$app/state';
	import { Button } from '@glyphx/ui/button';
	import { Logo } from '@glyphx/ui/logo';
	import { Toaster } from '@glyphx/ui/sonner';
	import {
		IconArrowLeft,
		IconBolt,
		IconInfoCircle,
		IconPencil,
		IconPlug,
		IconSettings
	} from '@tabler/icons-svelte';

	let { children } = $props();

	// Left-nav sections. Cloud / sync integration will slot in here later.
	const nav = [
		{ href: '/settings/general', label: 'General', icon: IconSettings },
		{ href: '/settings/editor', label: 'Editor', icon: IconPencil },
		{ href: '/settings/engine', label: 'Engine', icon: IconBolt },
		{ href: '/settings/integrations', label: 'Integrations', icon: IconPlug },
		{ href: '/settings/about', label: 'About', icon: IconInfoCircle }
	] as const;
	const isActive = (href: string) => page.url.pathname === href;
</script>

<svelte:head><title>Settings · GlyphX</title></svelte:head>

<div class="bg-background text-foreground flex h-dvh flex-col overflow-hidden">
	<header class="border-border flex h-14 shrink-0 items-center gap-3 border-b px-5">
		<Button
			variant="ghost"
			size="icon-sm"
			title="Back to projects"
			aria-label="Back to projects"
			onclick={() => goto(resolve('/'))}
		>
			<IconArrowLeft size={16} />
		</Button>
		<Logo size={22} text={false} />
		<h1 class="font-display text-base tracking-tight">Settings</h1>
	</header>

	<div class="min-h-0 flex-1 overflow-auto">
		<div class="mx-auto flex w-full max-w-[860px] gap-8 px-6 py-8">
			<!-- Section nav -->
			<nav class="w-44 shrink-0" aria-label="Settings sections">
				<ul class="flex flex-col gap-0.5">
					{#each nav as item (item.href)}
						{@const Icon = item.icon}
						<li>
							<a
								href={resolve(item.href)}
								aria-current={isActive(item.href) ? 'page' : undefined}
								class="flex items-center gap-2 rounded-md px-2.5 py-1.5 text-[13px] transition-colors {isActive(
									item.href
								)
									? 'bg-muted text-foreground font-medium'
									: 'text-muted-foreground hover:bg-muted/60 hover:text-foreground'}"
							>
								<Icon size={16} class="shrink-0" />
								{item.label}
							</a>
						</li>
					{/each}
				</ul>
			</nav>

			<!-- Section content -->
			<main class="min-w-0 flex-1">
				{@render children()}
			</main>
		</div>
	</div>
</div>

<Toaster />
