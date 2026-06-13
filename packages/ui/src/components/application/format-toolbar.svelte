<script lang="ts">
	import { Button } from '@glyph/ui/button';
	import { ButtonGroup } from '@glyph/ui/button-group';
	import {
		DropdownMenu,
		DropdownMenuContent,
		DropdownMenuItem,
		DropdownMenuSeparator,
		DropdownMenuShortcut,
		DropdownMenuTrigger
	} from '@glyph/ui/dropdown-menu';
	import {
		IconBold,
		IconChevronDown,
		IconHeading,
		IconItalic,
		IconList,
		IconMath,
		IconMathFunction,
		IconMathSymbols,
		IconPlus,
		IconTypography,
		IconUnderline
	} from '@tabler/icons-svelte';

	/**
	 * FormatToolbar — a rich LaTeX formatting bar. The most common actions are
	 * direct buttons; the rest are grouped into portaled dropdown menus (same
	 * chrome as the file-tree menus). Each menu item shows the LaTeX command it
	 * inserts. Acts on the selection via the CodeEditor imperative API.
	 */
	let {
		wrap,
		insert
	}: {
		wrap?: (before: string, after?: string) => void;
		insert?: (text: string) => void;
	} = $props();

	const w = (before: string, after = '') => () => wrap?.(before, after);
	const i = (text: string) => () => insert?.(text);

	type Cmd = { label: string; hint?: string; run: () => void };
	type ButtonCmd = { icon: typeof IconBold; label: string; run: () => void };
	type Cluster =
		| { kind: 'group'; actions: ButtonCmd[] }
		| { kind: 'menu'; icon: typeof IconBold; label: string; items: (Cmd | 'sep')[] };

	const clusters: Cluster[] = [
		{
			kind: 'group',
			actions: [
				{ icon: IconBold, label: 'Bold', run: w('\\textbf{', '}') },
				{ icon: IconItalic, label: 'Italic', run: w('\\textit{', '}') },
				{ icon: IconUnderline, label: 'Underline', run: w('\\underline{', '}') }
			]
		},
		{
			kind: 'menu',
			icon: IconTypography,
			label: 'Text style',
			items: [
				{ label: 'Emphasis', hint: '\\emph{}', run: w('\\emph{', '}') },
				{ label: 'Small caps', hint: '\\textsc{}', run: w('\\textsc{', '}') },
				{ label: 'Monospace', hint: '\\texttt{}', run: w('\\texttt{', '}') },
				{ label: 'Sans serif', hint: '\\textsf{}', run: w('\\textsf{', '}') },
				{ label: 'Strikethrough', hint: '\\sout{}', run: w('\\sout{', '}') },
				'sep',
				{ label: 'Superscript', hint: '\\textsuperscript{}', run: w('\\textsuperscript{', '}') },
				{ label: 'Subscript', hint: '\\textsubscript{}', run: w('\\textsubscript{', '}') }
			]
		},
		{
			kind: 'menu',
			icon: IconHeading,
			label: 'Heading',
			items: [
				{ label: 'Part', hint: '\\part{}', run: w('\\part{', '}') },
				{ label: 'Chapter', hint: '\\chapter{}', run: w('\\chapter{', '}') },
				{ label: 'Section', hint: '\\section{}', run: w('\\section{', '}') },
				{ label: 'Subsection', hint: '\\subsection{}', run: w('\\subsection{', '}') },
				{ label: 'Subsubsection', hint: '\\subsubsection{}', run: w('\\subsubsection{', '}') },
				{ label: 'Paragraph', hint: '\\paragraph{}', run: w('\\paragraph{', '}') }
			]
		},
		{
			kind: 'menu',
			icon: IconList,
			label: 'List',
			items: [
				{
					label: 'Bulleted list',
					hint: 'itemize',
					run: i('\\begin{itemize}\n  \\item \n\\end{itemize}\n')
				},
				{
					label: 'Numbered list',
					hint: 'enumerate',
					run: i('\\begin{enumerate}\n  \\item \n\\end{enumerate}\n')
				},
				{
					label: 'Description list',
					hint: 'description',
					run: i('\\begin{description}\n  \\item[Term] Description\n\\end{description}\n')
				}
			]
		},
		{
			kind: 'group',
			actions: [
				{ icon: IconMath, label: 'Inline math', run: w('$', '$') },
				{ icon: IconMathFunction, label: 'Display math', run: w('\\[\n  ', '\n\\]') }
			]
		},
		{
			kind: 'menu',
			icon: IconMathSymbols,
			label: 'Math',
			items: [
				{ label: 'Equation', hint: 'equation', run: i('\\begin{equation}\n  \n\\end{equation}\n') },
				{ label: 'Aligned', hint: 'align', run: i('\\begin{align}\n  \n\\end{align}\n') },
				'sep',
				{ label: 'Fraction', hint: '\\frac{}{}', run: w('\\frac{', '}{}') },
				{ label: 'Square root', hint: '\\sqrt{}', run: w('\\sqrt{', '}') },
				{ label: 'Summation', hint: '\\sum', run: i('\\sum_{i=1}^{n} ') },
				{ label: 'Product', hint: '\\prod', run: i('\\prod_{i=1}^{n} ') },
				{ label: 'Integral', hint: '\\int', run: i('\\int_{a}^{b} ') },
				{ label: 'Limit', hint: '\\lim', run: i('\\lim_{x \\to 0} ') },
				'sep',
				{
					label: 'Matrix',
					hint: 'pmatrix',
					run: i('\\begin{pmatrix}\n  a & b \\\\\n  c & d\n\\end{pmatrix}\n')
				},
				{ label: 'Cases', hint: 'cases', run: i('\\begin{cases}\n  \n\\end{cases}\n') }
			]
		},
		{
			kind: 'menu',
			icon: IconPlus,
			label: 'Insert',
			items: [
				{ label: 'Link', hint: '\\href{}{}', run: w('\\href{url}{', '}') },
				{ label: 'Footnote', hint: '\\footnote{}', run: w('\\footnote{', '}') },
				{ label: 'Citation', hint: '\\cite{}', run: w('\\cite{', '}') },
				{ label: 'Cross-reference', hint: '\\ref{}', run: w('\\ref{', '}') },
				{ label: 'Label', hint: '\\label{}', run: w('\\label{', '}') },
				'sep',
				{
					label: 'Figure',
					hint: 'figure',
					run: i(
						'\\begin{figure}[h]\n  \\centering\n  \\includegraphics[width=0.8\\linewidth]{}\n  \\caption{}\n  \\label{fig:}\n\\end{figure}\n'
					)
				},
				{
					label: 'Table',
					hint: 'tabular',
					run: i(
						'\\begin{table}[h]\n  \\centering\n  \\begin{tabular}{c c}\n    a & b \\\\\n    c & d\n  \\end{tabular}\n  \\caption{}\n  \\label{tab:}\n\\end{table}\n'
					)
				},
				{ label: 'Code block', hint: 'verbatim', run: i('\\begin{verbatim}\n\n\\end{verbatim}\n') },
				{ label: 'Block quote', hint: 'quote', run: i('\\begin{quote}\n\n\\end{quote}\n') }
			]
		}
	];

	const isSep = (x: Cmd | 'sep'): x is 'sep' => x === 'sep';
</script>

<div class="flex items-center gap-1.5" role="toolbar" aria-label="Formatting">
	{#each clusters as cluster, ci (ci)}
		{#if cluster.kind === 'group'}
			<ButtonGroup>
				{#each cluster.actions as a (a.label)}
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
		{:else}
			{@const Icon = cluster.icon}
			<DropdownMenu>
				<DropdownMenuTrigger>
					{#snippet child({ props })}
						<Button
							{...props}
							variant="outline"
							size="xs"
							class="gap-1 px-1.5"
							title={cluster.label}
							aria-label={cluster.label}
						>
							<Icon />
							<IconChevronDown class="size-3 opacity-50" />
						</Button>
					{/snippet}
				</DropdownMenuTrigger>
				<DropdownMenuContent align="start" class="w-56">
					{#each cluster.items as item, ii (ii)}
						{#if isSep(item)}
							<DropdownMenuSeparator />
						{:else}
							<DropdownMenuItem onSelect={item.run}>
								<span class="flex-1">{item.label}</span>
								{#if item.hint}<DropdownMenuShortcut class="font-mono">{item.hint}</DropdownMenuShortcut
									>{/if}
							</DropdownMenuItem>
						{/if}
					{/each}
				</DropdownMenuContent>
			</DropdownMenu>
		{/if}
	{/each}
</div>
