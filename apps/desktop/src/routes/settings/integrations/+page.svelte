<script lang="ts">
	import { EngineSettings } from '@glyph/ui/application';
	import { Button } from '@glyph/ui/button';
	import { toast } from '@glyph/ui/sonner';
	import { IconCloud } from '@tabler/icons-svelte';
	import { engineManager } from '$lib/engine';
	import { projectHost } from '$lib/project';

	async function addShellIntegration() {
		try {
			const msg = await projectHost.registerShellIntegration?.();
			if (msg) toast.success(msg);
		} catch (e) {
			toast.error(`Could not register shell integration — ${e}`);
		}
	}
</script>

<div class="flex max-w-xl flex-col gap-8">
	<div>
		<h2 class="text-lg font-medium tracking-tight">Integrations</h2>
		<p class="text-muted-foreground mt-1 text-sm">
			The LaTeX engine, OS integration, and (later) cloud sync.
		</p>
	</div>

	<section class="flex flex-col gap-3">
		<span class="text-foreground text-sm font-medium">LaTeX engine</span>
		<EngineSettings engine={engineManager} />
	</section>

	<div class="bg-border h-px"></div>

	<section class="flex flex-col gap-2">
		<span class="text-foreground text-sm font-medium">System integration</span>
		<p class="text-muted-foreground text-xs leading-relaxed">
			Add an “Open with Glyph” entry to the folder right-click menu. (.tex and .glyx files are
			associated by the installer.)
		</p>
		<Button variant="outline" size="sm" class="self-start" onclick={addShellIntegration}>
			Add “Open with Glyph”
		</Button>
	</section>

	<div class="bg-border h-px"></div>

	<section class="flex flex-col gap-2">
		<div class="flex items-center gap-2">
			<span class="text-foreground text-sm font-medium">Cloud sync</span>
			<span
				class="bg-muted text-muted-foreground rounded-full px-2 py-0.5 text-[10px] font-medium tracking-wide uppercase"
			>
				Coming soon
			</span>
		</div>
		<div
			class="border-border text-muted-foreground flex items-center gap-3 rounded-lg border border-dashed px-4 py-5 text-sm"
		>
			<IconCloud size={20} class="shrink-0 opacity-70" />
			<p class="leading-relaxed">
				Optional end-to-end encrypted sync across your devices. Glyph stays local-first — this will
				always be opt-in.
			</p>
		</div>
	</section>
</div>
