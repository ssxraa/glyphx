<script lang="ts">
	import { Logo } from '@glyph/ui/logo';
</script>

<div class="flex max-w-xl flex-col gap-8">
	<div>
		<h2 class="text-lg font-medium tracking-tight">About</h2>
		<p class="text-muted-foreground mt-1 text-sm">What Glyph is, and what it promises.</p>
	</div>

	<section class="flex items-center gap-4">
		<Logo text={false} badge size={48} />
		<div>
			<p class="text-foreground text-base font-medium">Glyph</p>
			<p class="text-muted-foreground text-sm">Local-first LaTeX editor · v0.1.0</p>
		</div>
	</section>

	<section class="text-muted-foreground flex flex-col gap-3 text-sm leading-relaxed">
		<p>
			Glyph compiles real LaTeX on your machine with Tectonic — nothing is uploaded, and it works
			fully offline. Your documents stay on disk, in plain files you own.
		</p>
		<p>
			Open any folder as a project, just like a normal LaTeX setup: a main file plus whatever you
			<code class="bg-muted rounded px-1 py-0.5 text-[12px]">\input</code> or include.
		</p>
	</section>
</div>
