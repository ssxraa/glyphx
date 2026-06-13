<script lang="ts">
	import { Button } from '@glyph/ui/button';
	import { ButtonGroup } from '@glyph/ui/button-group';
	import {
		IconBold,
		IconItalic,
		IconUnderline,
		IconMath,
		IconCode,
		IconHeading,
		IconList,
		IconLink
	} from '@tabler/icons-svelte';

	/**
	 * FormatToolbar — LaTeX formatting controls. Lives in the editor panel header
	 * and acts on the active selection via the CodeEditor imperative API. Built
	 * from the shared Button + ButtonGroup: each logical cluster is a connected
	 * segmented group (consistent with the app's other grouped chrome).
	 */
	let {
		wrap,
		insert
	}: {
		wrap?: (before: string, after?: string) => void;
		insert?: (text: string) => void;
	} = $props();

	type Action = { icon: typeof IconBold; label: string; run: () => void };

	// Grouped: text emphasis · math/code · structure.
	const groups: Action[][] = [
		[
			{ icon: IconBold, label: 'Bold', run: () => wrap?.('\\textbf{', '}') },
			{ icon: IconItalic, label: 'Italic', run: () => wrap?.('\\textit{', '}') },
			{ icon: IconUnderline, label: 'Underline', run: () => wrap?.('\\underline{', '}') }
		],
		[
			{ icon: IconMath, label: 'Inline math', run: () => wrap?.('$', '$') },
			{ icon: IconCode, label: 'Monospace', run: () => wrap?.('\\texttt{', '}') }
		],
		[
			{ icon: IconHeading, label: 'Section', run: () => wrap?.('\\section{', '}') },
			{
				icon: IconList,
				label: 'Itemize',
				run: () => insert?.('\\begin{itemize}\n  \\item \n\\end{itemize}\n')
			},
			{ icon: IconLink, label: 'Link', run: () => wrap?.('\\href{url}{', '}') }
		]
	];
</script>

<div class="flex items-center gap-1.5" role="toolbar" aria-label="Formatting">
	{#each groups as group, gi (gi)}
		<ButtonGroup>
			{#each group as a (a.label)}
				{@const Icon = a.icon}
				<Button
					variant="outline"
					size="icon-xs"
					title={a.label}
					aria-label={a.label}
					onclick={a.run}
				>
					<Icon />
				</Button>
			{/each}
		</ButtonGroup>
	{/each}
</div>
