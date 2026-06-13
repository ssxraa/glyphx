/**
 * JetBrains-matched CodeMirror 6 themes.
 *
 * Two faithful ports of the JetBrains IDE colour schemes:
 *   - `jetbrainsDark`  → Darcula
 *   - `jetbrainsLight` → IntelliJ Light
 *
 * These are intentionally the *real* JetBrains palettes (cool grey / true
 * white), NOT the warm Clay app chrome — the editor surface keeps its own
 * IDE identity, as specified.
 */
import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";
import type { Extension } from "@codemirror/state";

/* ------------------------------------------------------------------ Darcula */

const darcula = {
	bg: "#2b2b2b",
	fg: "#a9b7c6",
	caret: "#bbbbbb",
	selection: "#214283",
	selectionMatch: "#32593d",
	// Gutter blends into the editor surface; the current line number pops in the
	// brand accent, and the active line gets a subtle neutral lift.
	gutterBg: "#2b2b2b",
	gutterFg: "#646b75",
	gutterActiveFg: "#34d399",
	lineHighlight: "rgba(255, 255, 255, 0.055)",
	// syntax
	keyword: "#cc7832",
	comment: "#808080",
	string: "#6a8759",
	number: "#6897bb",
	func: "#ffc66d",
	constant: "#9876aa",
	meta: "#bbb529",
	invalid: "#bc3f3c",
} as const;

const darculaTheme = EditorView.theme(
	{
		"&": { color: darcula.fg, backgroundColor: darcula.bg },
		".cm-content": { caretColor: darcula.caret },
		".cm-cursor, .cm-dropCursor": { borderLeftColor: darcula.caret },
		"&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
			{ backgroundColor: darcula.selection },
		".cm-selectionMatch": { backgroundColor: darcula.selectionMatch },
		".cm-activeLine": { backgroundColor: darcula.lineHighlight },
		".cm-activeLineGutter": {
			backgroundColor: darcula.lineHighlight,
			color: darcula.gutterActiveFg,
		},
		".cm-gutters": {
			backgroundColor: darcula.gutterBg,
			color: darcula.gutterFg,
			border: "none",
		},
		".cm-matchingBracket, .cm-nonmatchingBracket": {
			backgroundColor: "#3b514d",
			color: "inherit",
			outline: "1px solid #5b6b65",
		},
		".cm-tooltip": {
			backgroundColor: "#3c3f41",
			border: "1px solid #4b4b4b",
			color: darcula.fg,
		},
		".cm-foldPlaceholder": {
			backgroundColor: "#3c3f41",
			border: "none",
			color: "#8a8a8a",
		},
	},
	{ dark: true },
);

const darculaHighlight = HighlightStyle.define([
	{ tag: [t.keyword, t.moduleKeyword, t.controlKeyword, t.operatorKeyword], color: darcula.keyword },
	{ tag: [t.comment, t.lineComment, t.blockComment, t.docComment], color: darcula.comment, fontStyle: "italic" },
	{ tag: [t.string, t.special(t.string), t.character], color: darcula.string },
	{ tag: [t.number, t.integer, t.float], color: darcula.number },
	{ tag: [t.bool, t.null, t.atom, t.constant(t.name)], color: darcula.constant },
	{ tag: [t.function(t.variableName), t.function(t.propertyName), t.macroName, t.labelName], color: darcula.func },
	{ tag: [t.tagName], color: darcula.keyword },
	{ tag: [t.meta, t.annotation, t.processingInstruction], color: darcula.meta },
	{ tag: [t.variableName, t.propertyName, t.attributeName], color: darcula.fg },
	{ tag: [t.typeName, t.className, t.namespace], color: darcula.fg },
	{ tag: [t.operator, t.derefOperator, t.bracket, t.brace, t.paren, t.punctuation], color: darcula.fg },
	{ tag: [t.heading], color: darcula.keyword, fontWeight: "bold" },
	{ tag: t.strong, fontWeight: "bold" },
	{ tag: t.emphasis, fontStyle: "italic" },
	{ tag: [t.link, t.url], color: darcula.number, textDecoration: "underline" },
	{ tag: t.invalid, color: darcula.invalid },
]);

export const jetbrainsDark: Extension = [
	darculaTheme,
	syntaxHighlighting(darculaHighlight),
];

/* ----------------------------------------------------------- IntelliJ Light */

const light = {
	bg: "#ffffff",
	fg: "#080808",
	caret: "#000000",
	selection: "#a6d2ff",
	selectionMatch: "#cce8ff",
	// Neutral (cool) active-line tint to match Glyph's near-white surface — not
	// the warm cream of stock IntelliJ — with the current line number in accent.
	gutterBg: "#ffffff",
	gutterFg: "#9b9ba3",
	gutterActiveFg: "#0d9373",
	lineHighlight: "rgba(10, 10, 12, 0.04)",
	// syntax
	keyword: "#0033b3",
	comment: "#8c8c8c",
	string: "#067d17",
	number: "#1750eb",
	func: "#00627a",
	constant: "#871094",
	meta: "#9e880d",
	invalid: "#ff0000",
} as const;

const lightTheme = EditorView.theme(
	{
		"&": { color: light.fg, backgroundColor: light.bg },
		".cm-content": { caretColor: light.caret },
		".cm-cursor, .cm-dropCursor": { borderLeftColor: light.caret },
		"&.cm-focused > .cm-scroller > .cm-selectionLayer .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection":
			{ backgroundColor: light.selection },
		".cm-selectionMatch": { backgroundColor: light.selectionMatch },
		".cm-activeLine": { backgroundColor: light.lineHighlight },
		".cm-activeLineGutter": {
			backgroundColor: light.lineHighlight,
			color: light.gutterActiveFg,
		},
		".cm-gutters": {
			backgroundColor: light.gutterBg,
			color: light.gutterFg,
			border: "none",
		},
		".cm-matchingBracket, .cm-nonmatchingBracket": {
			backgroundColor: "#d4eaff",
			color: "inherit",
			outline: "1px solid #9cc8f5",
		},
		".cm-tooltip": {
			backgroundColor: "#f7f7f7",
			border: "1px solid #c9c9c9",
			color: light.fg,
		},
		".cm-foldPlaceholder": {
			backgroundColor: "#eeeeee",
			border: "none",
			color: "#888888",
		},
	},
	{ dark: false },
);

const lightHighlight = HighlightStyle.define([
	{ tag: [t.keyword, t.moduleKeyword, t.controlKeyword, t.operatorKeyword], color: light.keyword },
	{ tag: [t.comment, t.lineComment, t.blockComment, t.docComment], color: light.comment, fontStyle: "italic" },
	{ tag: [t.string, t.special(t.string), t.character], color: light.string },
	{ tag: [t.number, t.integer, t.float], color: light.number },
	{ tag: [t.bool, t.null, t.atom, t.constant(t.name)], color: light.constant },
	{ tag: [t.function(t.variableName), t.function(t.propertyName), t.macroName, t.labelName], color: light.func },
	{ tag: [t.tagName], color: light.keyword },
	{ tag: [t.meta, t.annotation, t.processingInstruction], color: light.meta },
	{ tag: [t.variableName, t.propertyName, t.attributeName], color: light.fg },
	{ tag: [t.typeName, t.className, t.namespace], color: light.fg },
	{ tag: [t.operator, t.derefOperator, t.bracket, t.brace, t.paren, t.punctuation], color: light.fg },
	{ tag: [t.heading], color: light.keyword, fontWeight: "bold" },
	{ tag: t.strong, fontWeight: "bold" },
	{ tag: t.emphasis, fontStyle: "italic" },
	{ tag: [t.link, t.url], color: light.number, textDecoration: "underline" },
	{ tag: t.invalid, color: light.invalid },
]);

export const jetbrainsLight: Extension = [
	lightTheme,
	syntaxHighlighting(lightHighlight),
];

export function jetbrainsTheme(mode: "light" | "dark"): Extension {
	return mode === "dark" ? jetbrainsDark : jetbrainsLight;
}
