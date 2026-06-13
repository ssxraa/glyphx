<script lang="ts">
	import { untrack } from "svelte";
	import {
		EditorView,
		keymap,
		lineNumbers,
		highlightActiveLine,
		highlightActiveLineGutter,
		highlightSpecialChars,
		drawSelection,
		dropCursor,
		rectangularSelection,
		crosshairCursor,
	} from "@codemirror/view";
	import { EditorState, Compartment } from "@codemirror/state";
	import { history, historyKeymap, defaultKeymap, indentWithTab } from "@codemirror/commands";
	import {
		indentOnInput,
		indentUnit,
		bracketMatching,
		foldGutter,
		foldKeymap,
	} from "@codemirror/language";
	import {
		closeBrackets,
		closeBracketsKeymap,
		autocompletion,
		completionKeymap,
	} from "@codemirror/autocomplete";

	import { jetbrainsTheme, latexLanguage, type LatexGrammar } from "@glyph/ui/editor";

	/**
	 * CodeEditor — the shared CodeMirror 6 surface (web + desktop).
	 *
	 * Dumb on purpose: theme / grammar / font come in as props; EditorShell
	 * (or any host) wires them to the settings store. Reconfiguration runs
	 * through Compartments so toggling the theme or swapping the LaTeX parser
	 * never re-mounts the view or loses cursor/scroll/history.
	 */
	let {
		value = $bindable(""),
		theme = "light" as "light" | "dark",
		grammar = "legacy" as LatexGrammar,
		fontSize = 13,
		fontFamily = "'JetBrains Mono Variable', 'JetBrains Mono', ui-monospace, monospace",
		lineWrapping = false,
		readonly = false,
		class: className = "",
		oncursor,
	}: {
		value?: string;
		theme?: "light" | "dark";
		grammar?: LatexGrammar;
		fontSize?: number;
		fontFamily?: string;
		lineWrapping?: boolean;
		readonly?: boolean;
		class?: string;
		/** Fires with the 1-based caret position whenever the selection moves. */
		oncursor?: (pos: { line: number; column: number }) => void;
	} = $props();

	let host = $state<HTMLDivElement>();
	let view = $state<EditorView>();

	const themeC = new Compartment();
	const langC = new Compartment();
	const fontC = new Compartment();
	const wrapC = new Compartment();
	const roC = new Compartment();

	// Font size + family live in a compartment so Settings can change them live.
	// Default is JetBrains Mono (per product spec); the host may pass another.
	const fontExtension = (size: number, family: string) =>
		EditorView.theme({
			"&": { fontSize: `${size}px` },
			".cm-scroller": { fontFamily: family },
			".cm-gutters": { fontFamily: family },
		});

	const baseTheme = EditorView.theme({
		"&": { height: "100%", backgroundColor: "transparent" },
		"&.cm-focused": { outline: "none" },
		".cm-scroller": { lineHeight: "1.6", overflow: "auto" },
		".cm-content": { padding: "12px 0" },
		".cm-lineNumbers .cm-gutterElement": { padding: "0 12px 0 8px" },
	});

	// Mount once. Initial prop values are read untracked so this effect does
	// not re-run (and re-create the view) when they change — the dedicated
	// effects below handle live reconfiguration.
	$effect(() => {
		const parent = host;
		if (!parent) return;

		const init = untrack(() => ({
			value,
			theme,
			grammar,
			fontSize,
			fontFamily,
			lineWrapping,
			readonly,
		}));

		const state = EditorState.create({
			doc: init.value,
			extensions: [
				lineNumbers(),
				highlightActiveLineGutter(),
				highlightSpecialChars(),
				history(),
				foldGutter(),
				drawSelection(),
				dropCursor(),
				EditorState.allowMultipleSelections.of(true),
				indentOnInput(),
				indentUnit.of("  "),
				bracketMatching(),
				closeBrackets(),
				autocompletion(),
				rectangularSelection(),
				crosshairCursor(),
				highlightActiveLine(),
				keymap.of([
					...closeBracketsKeymap,
					...defaultKeymap,
					...historyKeymap,
					...foldKeymap,
					...completionKeymap,
					indentWithTab,
				]),
				baseTheme,
				langC.of(latexLanguage(init.grammar)),
				themeC.of(jetbrainsTheme(init.theme)),
				fontC.of(fontExtension(init.fontSize, init.fontFamily)),
				wrapC.of(init.lineWrapping ? EditorView.lineWrapping : []),
				roC.of(EditorState.readOnly.of(init.readonly)),
				EditorView.updateListener.of((u) => {
					if (u.docChanged) value = u.state.doc.toString();
					if ((u.docChanged || u.selectionSet) && oncursor) {
						const head = u.state.selection.main.head;
						const line = u.state.doc.lineAt(head);
						oncursor({ line: line.number, column: head - line.from + 1 });
					}
				}),
			],
		});

		const v = new EditorView({ state, parent });
		view = v;
		return () => {
			v.destroy();
			if (view === v) view = undefined;
		};
	});

	// Live reconfiguration — each tracks exactly one prop.
	$effect(() => {
		const v = view;
		const mode = theme;
		if (v) v.dispatch({ effects: themeC.reconfigure(jetbrainsTheme(mode)) });
	});
	$effect(() => {
		const v = view;
		const g = grammar;
		if (v) v.dispatch({ effects: langC.reconfigure(latexLanguage(g)) });
	});
	$effect(() => {
		const v = view;
		const size = fontSize;
		const family = fontFamily;
		if (v) v.dispatch({ effects: fontC.reconfigure(fontExtension(size, family)) });
	});
	$effect(() => {
		const v = view;
		const wrap = lineWrapping;
		if (v) v.dispatch({ effects: wrapC.reconfigure(wrap ? EditorView.lineWrapping : []) });
	});
	$effect(() => {
		const v = view;
		const ro = readonly;
		if (v) v.dispatch({ effects: roC.reconfigure(EditorState.readOnly.of(ro)) });
	});

	// External value → editor (e.g. open a different file). Guarded so typing
	// (which writes `value` via the update listener) doesn't loop.
	$effect(() => {
		const v = view;
		const next = value;
		if (!v) return;
		if (next !== v.state.doc.toString()) {
			v.dispatch({ changes: { from: 0, to: v.state.doc.length, insert: next } });
		}
	});

	// --- Imperative API (accessed via bind:this from a toolbar, etc.) --------

	/** Wrap the current selection (or insert at the caret) with delimiters. */
	export function wrapSelection(before: string, after: string = before) {
		const v = view;
		if (!v) return;
		const range = v.state.selection.main;
		const selected = v.state.sliceDoc(range.from, range.to);
		v.dispatch({
			changes: { from: range.from, to: range.to, insert: `${before}${selected}${after}` },
			selection: {
				anchor: range.from + before.length,
				head: range.from + before.length + selected.length,
			},
			scrollIntoView: true,
		});
		v.focus();
	}

	/** Replace the current selection with `text` (snippets, environments). */
	export function insertText(text: string) {
		const v = view;
		if (!v) return;
		const range = v.state.selection.main;
		v.dispatch({
			changes: { from: range.from, to: range.to, insert: text },
			selection: { anchor: range.from + text.length },
			scrollIntoView: true,
		});
		v.focus();
	}

	export function focusEditor() {
		view?.focus();
	}

	/** Scroll to and place the caret on a 1-based line (SyncTeX reverse search). */
	export function goToLine(line: number) {
		const v = view;
		if (!v) return;
		const n = Math.max(1, Math.min(line, v.state.doc.lines));
		const l = v.state.doc.line(n);
		v.dispatch({
			selection: { anchor: l.from },
			effects: EditorView.scrollIntoView(l.from, { y: "center" }),
		});
		v.focus();
	}
</script>

<div bind:this={host} class="h-full min-h-0 {className}"></div>
