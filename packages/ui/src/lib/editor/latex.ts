/**
 * LaTeX language support — swappable parser backend.
 *
 *   - `legacy` → @codemirror/legacy-modes `stex` (battle-tested StreamLanguage)
 *   - `lezer`  → codemirror-lang-latex (Lezer grammar: richer tokens, folding)
 *
 * The active backend is a user setting (see settings.svelte.ts). Default is
 * `legacy`; `lezer` is opt-in. If the Lezer grammar ever fails to load we fall
 * back to legacy so the editor never breaks.
 */
import { StreamLanguage, type LanguageSupport } from "@codemirror/language";
import { stex } from "@codemirror/legacy-modes/mode/stex";
import { latex as lezerLatex } from "codemirror-lang-latex";
import type { Extension } from "@codemirror/state";

export type LatexGrammar = "legacy" | "lezer";

const legacy = (): Extension => StreamLanguage.define(stex);

const lezer = (): Extension | LanguageSupport =>
	lezerLatex({
		// Pure grammar: highlighting + structure only. The shell owns autocomplete,
		// linting and tooltips so the two backends behave identically.
		enableAutocomplete: false,
		enableLinting: false,
		enableTooltips: false,
		autoCloseTags: false,
		autoCloseBrackets: false,
	});

export function latexLanguage(grammar: LatexGrammar): Extension {
	if (grammar === "lezer") {
		try {
			return lezer() as Extension;
		} catch {
			return legacy();
		}
	}
	return legacy();
}
