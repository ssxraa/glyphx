<script lang="ts">
	import { Button } from '@glyphx/ui/button';
	import { Segmented } from '@glyphx/ui/segmented';
	import { Separator } from '@glyphx/ui/separator';
	import {
		AUTO_SAVE_LABELS,
		EDITOR_FONT_LABELS,
		settings,
		type AutoSaveMode,
		type EditorFont,
		type LatexGrammar
	} from '@glyphx/ui/settings';
	import { SettingsField } from '@glyphx/ui/settings-field';
	import { Switch } from '@glyphx/ui/switch';
	import { IconMinus, IconPlus } from '@tabler/icons-svelte';

	const grammarOpts: { value: LatexGrammar; label: string }[] = [
		{ value: 'legacy', label: 'stex' },
		{ value: 'lezer', label: 'lezer' }
	];
	const fontOpts = (Object.keys(EDITOR_FONT_LABELS) as EditorFont[]).map((id) => ({
		value: id,
		label: EDITOR_FONT_LABELS[id]
	}));
	const autoSaveOpts = (Object.keys(AUTO_SAVE_LABELS) as AutoSaveMode[]).map((id) => ({
		value: id,
		label: AUTO_SAVE_LABELS[id]
	}));
</script>

<div class="flex max-w-xl flex-col gap-8">
	<div>
		<h2 class="text-lg font-medium tracking-tight">Editor</h2>
		<p class="text-muted-foreground mt-1 text-sm">Highlighting, typeface, and compile behaviour.</p>
	</div>

	<SettingsField label="LaTeX grammar">
		<Segmented
			options={grammarOpts}
			value={settings.grammar}
			onValueChange={(v) => (settings.grammar = v)}
			size="sm"
			aria-label="LaTeX grammar"
		/>
	</SettingsField>

	<SettingsField label="Editor font">
		<Segmented
			options={fontOpts}
			value={settings.font}
			onValueChange={(v) => (settings.font = v)}
			size="sm"
			aria-label="Editor font"
		/>
	</SettingsField>

	<SettingsField label="Editor font size" layout="row">
		<div class="flex items-center gap-1.5">
			<Button
				variant="outline"
				size="icon-xs"
				aria-label="Decrease font size"
				disabled={settings.fontSize <= 10}
				onclick={() => (settings.fontSize = Math.max(10, settings.fontSize - 1))}
			>
				<IconMinus size={15} />
			</Button>
			<span class="text-foreground w-10 text-center text-sm tabular-nums"
				>{settings.fontSize}px</span
			>
			<Button
				variant="outline"
				size="icon-xs"
				aria-label="Increase font size"
				disabled={settings.fontSize >= 24}
				onclick={() => (settings.fontSize = Math.min(24, settings.fontSize + 1))}
			>
				<IconPlus size={15} />
			</Button>
		</div>
	</SettingsField>

	<Separator />

	<SettingsField label="Line wrapping" layout="row">
		<Switch
			checked={settings.lineWrapping}
			onCheckedChange={(v) => (settings.lineWrapping = v)}
			aria-label="Line wrapping"
		/>
	</SettingsField>

	<SettingsField
		label="Auto save"
		description="When edits are written to disk: off (only on ⌘/Ctrl+S), after a short delay, or when the editor loses focus. The preview always renders the last saved version."
	>
		<Segmented
			options={autoSaveOpts}
			value={settings.autoSave}
			onValueChange={(v) => (settings.autoSave = v)}
			size="sm"
			aria-label="Auto save"
		/>
	</SettingsField>

	<SettingsField
		label="Live compile"
		description="Recompile automatically whenever a file is saved. With auto save off, the preview refreshes on save (⌘/Ctrl+S); turn auto save to “After delay” for a live, type-and-see preview."
		layout="row"
	>
		<Switch
			checked={settings.autoCompile}
			onCheckedChange={(v) => (settings.autoCompile = v)}
			aria-label="Live compile"
		/>
	</SettingsField>

	<SettingsField
		label="Shell escape"
		description="Allow \\write18 so packages like minted / gnuplot can run external tools. Off by default — it lets a document run system commands, so only enable it for documents you trust."
		layout="row"
	>
		<Switch
			checked={settings.shellEscape}
			onCheckedChange={(v) => (settings.shellEscape = v)}
			aria-label="Shell escape"
		/>
	</SettingsField>
</div>
