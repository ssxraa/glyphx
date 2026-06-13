<script lang="ts">
	import { Button } from '@glyph/ui/button';
	import { Segmented } from '@glyph/ui/segmented';
	import {
		EDITOR_FONT_LABELS,
		settings,
		type EditorFont,
		type LatexGrammar
	} from '@glyph/ui/settings';
	import { Switch } from '@glyph/ui/switch';
	import { IconMinus, IconPlus } from '@tabler/icons-svelte';

	const grammarOpts: { value: LatexGrammar; label: string }[] = [
		{ value: 'legacy', label: 'stex' },
		{ value: 'lezer', label: 'lezer' }
	];
	const fontOpts = (Object.keys(EDITOR_FONT_LABELS) as EditorFont[]).map((id) => ({
		value: id,
		label: EDITOR_FONT_LABELS[id]
	}));
</script>

<div class="flex max-w-xl flex-col gap-8">
	<div>
		<h2 class="text-lg font-medium tracking-tight">Editor</h2>
		<p class="text-muted-foreground mt-1 text-sm">Highlighting, typeface, and compile behaviour.</p>
	</div>

	<section class="flex flex-col gap-2">
		<span class="text-foreground text-sm font-medium">LaTeX grammar</span>
		<Segmented
			options={grammarOpts}
			value={settings.grammar}
			onValueChange={(v) => (settings.grammar = v)}
			size="sm"
			aria-label="LaTeX grammar"
		/>
	</section>

	<section class="flex flex-col gap-2">
		<span class="text-foreground text-sm font-medium">Editor font</span>
		<Segmented
			options={fontOpts}
			value={settings.font}
			onValueChange={(v) => (settings.font = v)}
			size="sm"
			aria-label="Editor font"
		/>
	</section>

	<section class="flex items-center justify-between gap-3">
		<span class="text-foreground text-sm font-medium">Editor font size</span>
		<div class="flex items-center gap-1.5">
			<Button
				variant="outline"
				size="icon-sm"
				aria-label="Decrease font size"
				disabled={settings.fontSize <= 10}
				onclick={() => (settings.fontSize = Math.max(10, settings.fontSize - 1))}
			>
				<IconMinus size={15} />
			</Button>
			<span class="text-foreground w-10 text-center text-sm tabular-nums">{settings.fontSize}px</span>
			<Button
				variant="outline"
				size="icon-sm"
				aria-label="Increase font size"
				disabled={settings.fontSize >= 24}
				onclick={() => (settings.fontSize = Math.min(24, settings.fontSize + 1))}
			>
				<IconPlus size={15} />
			</Button>
		</div>
	</section>

	<div class="bg-border h-px"></div>

	<label class="flex cursor-pointer items-center justify-between gap-3">
		<span class="text-foreground text-sm font-medium">Line wrapping</span>
		<Switch
			checked={settings.lineWrapping}
			onCheckedChange={(v) => (settings.lineWrapping = v)}
			aria-label="Line wrapping"
		/>
	</label>

	<label class="flex cursor-pointer items-center justify-between gap-3">
		<div class="flex flex-col">
			<span class="text-foreground text-sm font-medium">Live compile</span>
			<span class="text-muted-foreground text-xs">Recompile automatically as you type.</span>
		</div>
		<Switch
			checked={settings.autoCompile}
			onCheckedChange={(v) => (settings.autoCompile = v)}
			aria-label="Live compile"
		/>
	</label>
</div>
