<script lang="ts">
	import { EngineSettings } from '@glyph/ui/application';
	import { SettingsField } from '@glyph/ui/settings-field';
	import { engineManager } from '$lib/engine';
</script>

<div class="flex max-w-xl flex-col gap-8">
	<div>
		<h2 class="text-lg font-medium tracking-tight">Engine</h2>
		<p class="text-muted-foreground mt-1 text-sm">
			Choose and update the LaTeX engine. Glyph ships with Tectonic — no install needed.
		</p>
	</div>

	<SettingsField
		label="Tectonic versions"
		description="Pin a specific build, or switch to Nightly for newer icon-font support. Download or remove versions any time — no app rebuild required."
	>
		<EngineSettings engine={engineManager} />
	</SettingsField>
</div>
