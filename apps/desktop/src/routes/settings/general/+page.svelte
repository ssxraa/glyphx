<script lang="ts">
	import { Segmented } from '@glyphx/ui/segmented';
	import { SettingsField } from '@glyphx/ui/settings-field';
	import { settings, type Appearance } from '@glyphx/ui/settings';

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

	<SettingsField label="Appearance" description="Follow the system theme, or pick light / dark.">
		<Segmented
			options={appearanceOpts}
			value={settings.appearance}
			onValueChange={(v) => (settings.appearance = v)}
			size="sm"
			aria-label="Appearance"
		/>
	</SettingsField>
</div>
