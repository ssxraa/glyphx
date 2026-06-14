<script lang="ts">
	import { resolve } from '$app/paths';
	import { Logo } from '@glyphx/ui/logo';
	import { ThemeToggle } from '@glyphx/ui/theme-toggle';
	import { FloatingMenu } from '$lib/motion-core';

	// Shared marketing nav, built on Motion Core's FloatingMenu: a floating bar
	// that expands into a full overlay with staggered split-text link reveals.
	// Section links point back to the home page so they work from /download too.
	const home = resolve('/');
	const repo = 'https://github.com/kanakkholwal/glyphx';

	const menuGroups = [
		{
			title: 'Product',
			variant: 'muted' as const,
			links: [
				{ label: 'Features', href: `${home}#features` },
				{ label: 'Compare', href: `${home}#compare` },
				{ label: 'FAQ', href: `${home}#faq` }
			]
		},
		{
			title: 'Get GlyphX',
			variant: 'default' as const,
			links: [
				{ label: 'Download', href: resolve('/download') },
				{ label: 'Open the editor', href: resolve('/editor') }
			]
		},
		{
			title: 'Project',
			variant: 'muted' as const,
			links: [
				{ label: 'Source code', href: repo },
				{ label: 'Releases', href: `${repo}/releases` }
			]
		}
	];
</script>

<FloatingMenu
	{menuGroups}
	primaryButton={{ label: 'Download', href: resolve('/download') }}
	secondaryButton={{ label: 'Open the editor', href: resolve('/editor') }}
>
	{#snippet logo()}
		<Logo href={home} size={26} class="text-base tracking-tight" />
	{/snippet}
	{#snippet actions()}
		<ThemeToggle />
	{/snippet}
</FloatingMenu>
