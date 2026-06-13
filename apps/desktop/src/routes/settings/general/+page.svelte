<script lang="ts">
	import { Segmented } from '@glyph/ui/segmented';
	import { settings, type Appearance } from '@glyph/ui/settings';

	const appearanceOpts: { value: Appearance; label: string }[] = [
		{ value: 'light', label: 'Light' },
		{ value: 'dark', label: 'Dark' },
		{ value: 'system', label: 'System' }
	];
</script>

<div class="flex max-w-xl flex-col gap-8">
	<div>
		<h2 class="text-lg font-medium tracking-tight">General</h2>
		<p class="text-muted-foreground mt-1 text-sm">Appearance and app-wide preferences.</p>
	</div>

	<section class="flex flex-col gap-2">
		<span class="text-foreground text-sm font-medium">Appearance</span>
		<Segmented
			options={appearanceOpts}
			value={settings.appearance}
			onValueChange={(v) => (settings.appearance = v)}
			size="sm"
			aria-label="Appearance"
		/>
		<p class="text-muted-foreground text-xs">Follow the system theme, or pick light / dark.</p>
	</section>
</div>
