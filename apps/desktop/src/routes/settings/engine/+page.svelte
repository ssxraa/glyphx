<script lang="ts">
	import { EngineSettings } from '@glyph/ui/application';
	import { SettingsField } from '@glyph/ui/settings-field';
	import { engineManager } from '$lib/engine';
</script>

<div class="flex max-w-xl flex-col gap-8">
	<div>
		<h2 class="text-lg font-medium tracking-tight">Engine</h2>
		<p class="text-muted-foreground mt-1 text-sm">
			Choose how Glyph compiles. Bundled Tectonic needs no install; System TeX uses your local
			TeX Live / MiKTeX for documents Tectonic can't render.
		</p>
	</div>

	<SettingsField
		label="LaTeX engine"
		description="Switch engines, pin a Tectonic build (or Nightly for newer icon-font support), or compile with your local TeX install. Changes apply to the next compile."
	>
		<EngineSettings engine={engineManager} />
	</SettingsField>
</div>
