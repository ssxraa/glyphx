// Browser LaTeX compilation via the SwiftLaTeX PdfTeX WASM engine
// (self-hosted in /static/swiftlatex). The engine runs in a Web Worker and
// fetches TeX packages on demand from SwiftLaTeX's public package server, so
// the first use of a package needs network; everything else is in-browser.

export type CompileOutcome = { pdf?: string; log?: string; error?: string };

/* eslint-disable @typescript-eslint/no-explicit-any */

const ENGINE_SRC = '/swiftlatex/PdfTeXEngine.js';

let enginePromise: Promise<any> | null = null;

/** Load PdfTeXEngine.js as a classic script; it exposes `window.exports.PdfTeXEngine`. */
function loadEngineClass(): Promise<any> {
	const w = window as any;
	if (w.__PdfTeXEngineClass) return Promise.resolve(w.__PdfTeXEngineClass);

	return new Promise((resolve, reject) => {
		const script = document.createElement('script');
		script.src = ENGINE_SRC;
		script.async = true;
		script.onload = () => {
			const cls = (window as any).exports?.PdfTeXEngine;
			if (cls) {
				w.__PdfTeXEngineClass = cls;
				resolve(cls);
			} else {
				reject(new Error('SwiftLaTeX engine loaded but PdfTeXEngine was not found.'));
			}
		};
		script.onerror = () => reject(new Error('Failed to load the SwiftLaTeX engine.'));
		document.head.appendChild(script);
	});
}

/** Lazily create + boot a single engine instance (one compile at a time). */
function getEngine(): Promise<any> {
	if (!enginePromise) {
		enginePromise = (async () => {
			const PdfTeXEngine = await loadEngineClass();
			const engine = new PdfTeXEngine();
			await engine.loadEngine();
			return engine;
		})().catch((e) => {
			enginePromise = null; // allow retry after a failed boot
			throw e;
		});
	}
	return enginePromise;
}

function bytesToBase64(bytes: Uint8Array): string {
	let binary = '';
	const chunk = 0x8000;
	for (let i = 0; i < bytes.length; i += chunk) {
		binary += String.fromCharCode(...bytes.subarray(i, i + chunk));
	}
	return btoa(binary);
}

export async function compileLatex(source: string): Promise<CompileOutcome> {
	if (typeof window === 'undefined') {
		return { error: 'Compilation runs in the browser.' };
	}
	try {
		const engine = await getEngine();
		engine.writeMemFSFile('main.tex', source);
		engine.setEngineMainFile('main.tex');
		const result = await engine.compileLaTeX();
		const log: string = result?.log ?? '';
		if (result?.status === 0 && result?.pdf) {
			return { pdf: bytesToBase64(new Uint8Array(result.pdf)), log };
		}
		return { log, error: 'LaTeX compilation failed.' };
	} catch (e) {
		return { error: String(e) };
	}
}
