<script lang="ts">
	import { EditorView, lineNumbers } from '@codemirror/view';
	import { EditorState } from '@codemirror/state';
	import { MergeView, unifiedMergeView } from '@codemirror/merge';
	import { jetbrainsTheme, latexLanguage, type LatexGrammar } from '@glyphx/ui/editor';

	/**
	 * DiffView — read-only diff of two texts, built on `@codemirror/merge`.
	 * `mode` switches between VS Code-style side-by-side (`MergeView`) and inline /
	 * unified (`unifiedMergeView`). Theme / font / grammar mirror the editor so a
	 * diff reads like the document it came from. Rebuilt whenever a prop changes.
	 */
	let {
		original = '',
		modified = '',
		mode = 'side' as 'side' | 'inline',
		theme = 'light' as 'light' | 'dark',
		grammar = 'legacy' as LatexGrammar,
		language = 'latex' as 'latex' | 'markdown' | 'plain',
		fontSize = 13,
		fontFamily = "'JetBrains Mono Variable', 'JetBrains Mono', ui-monospace, monospace"
	}: {
		original?: string;
		modified?: string;
		mode?: 'side' | 'inline';
		theme?: 'light' | 'dark';
		grammar?: LatexGrammar;
		language?: 'latex' | 'markdown' | 'plain';
		fontSize?: number;
		fontFamily?: string;
	} = $props();

	let host = $state<HTMLDivElement>();

	const langExtension = (lang: 'latex' | 'markdown' | 'plain', g: LatexGrammar) =>
		lang === 'latex' ? latexLanguage(g) : [];

	const fontExtension = (size: number, family: string) =>
		EditorView.theme({
			'&': { fontSize: `${size}px` },
			'.cm-scroller': { fontFamily: family },
			'.cm-gutters': { fontFamily: family }
		});

	const heightTheme = EditorView.theme({
		'&': { height: '100%', backgroundColor: 'transparent' },
		'.cm-scroller': { lineHeight: '1.6', overflow: 'auto' }
	});

	// Rebuild the view whenever any input changes (merge views can't be live
	// reconfigured the way the editor's compartments are).
	$effect(() => {
		const parent = host;
		if (!parent) return;
		const o = original;
		const m = modified;
		const md = mode;
		const common = [
			lineNumbers(),
			EditorView.editable.of(false),
			EditorState.readOnly.of(true),
			heightTheme,
			jetbrainsTheme(theme),
			langExtension(language, grammar),
			fontExtension(fontSize, fontFamily)
		];

		if (md === 'side') {
			const view = new MergeView({
				a: { doc: o, extensions: common },
				b: { doc: m, extensions: common },
				parent,
				gutter: true,
				highlightChanges: true,
				collapseUnchanged: { margin: 3, minSize: 4 }
			});
			return () => view.destroy();
		}

		const view = new EditorView({
			parent,
			state: EditorState.create({
				doc: m,
				extensions: [...common, unifiedMergeView({ original: o, mergeControls: false })]
			})
		});
		return () => view.destroy();
	});
</script>

<div bind:this={host} class="h-full min-h-0 w-full overflow-auto text-[13px]"></div>

<style>
	/* --- Side-by-side: make the panes fill the available height instead of
	   sitting at the top (MergeView lays editors out at content height). --- */
	:global(.cm-mergeView) {
		display: flex;
		flex-direction: column;
		min-height: 100%;
	}
	:global(.cm-mergeViewEditors) {
		flex: 1 1 auto;
		min-height: 0;
	}
	:global(.cm-mergeViewEditor) {
		display: flex;
		flex-direction: column;
		min-height: 0;
	}
	:global(.cm-mergeViewEditor > .cm-editor) {
		flex-grow: 1;
	}

	/* --- Clearer add/remove colors (semantic tokens, never hardcoded). --- */
	/* removed lines → red tint */
	:global(.cm-merge-a .cm-changedLine),
	:global(.cm-deletedChunk),
	:global(.cm-deletedLine) {
		background-color: color-mix(in oklab, var(--color-destructive) 14%, transparent) !important;
	}
	/* added / changed lines → green tint */
	:global(.cm-merge-b .cm-changedLine),
	:global(.cm-insertedLine),
	:global(.cm-inlineChangedLine) {
		background-color: color-mix(in oklab, var(--color-success) 14%, transparent) !important;
	}
	/* the precise text that changed within a line → stronger tint */
	:global(.cm-merge-a .cm-changedText),
	:global(.cm-deletedChunk .cm-deletedText) {
		background: color-mix(in oklab, var(--color-destructive) 30%, transparent) !important;
	}
	:global(.cm-merge-b .cm-changedText) {
		background: color-mix(in oklab, var(--color-success) 30%, transparent) !important;
	}

	/* --- Change gutter: show +/− glyphs instead of a thin colored bar. --- */
	:global(.cm-changeGutter) {
		width: 1.2em !important;
		padding-left: 0 !important;
	}
	:global(.cm-changeGutter .cm-gutterElement) {
		text-align: center;
		font-weight: 700;
	}
	/* removed → red − */
	:global(.cm-merge-a .cm-changedLineGutter),
	:global(.cm-deletedLineGutter) {
		background: transparent !important;
		color: var(--color-destructive);
	}
	:global(.cm-merge-a .cm-changedLineGutter)::after,
	:global(.cm-deletedLineGutter)::after {
		content: "−";
	}
	/* added / changed → green + */
	:global(.cm-merge-b .cm-changedLineGutter),
	:global(.cm-inlineChangedLineGutter) {
		background: transparent !important;
		color: var(--color-success);
	}
	:global(.cm-merge-b .cm-changedLineGutter)::after,
	:global(.cm-inlineChangedLineGutter)::after {
		content: "+";
	}
</style>
