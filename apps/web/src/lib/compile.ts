// Browser LaTeX compilation via the SwiftLaTeX PdfTeX WASM engine
// (self-hosted in /static/swiftlatex). The engine runs in a Web Worker and
// fetches TeX packages on demand from a TeX Live "on-demand" server, so the
// first use of a package needs network; everything else is in-browser.
//
// The engine's baked-in default server (texlive2.swiftlatex.com) is offline, so
// we repoint it. We try, in order:
//   1. PUBLIC_TEXLIVE_ENDPOINT — your own self-hosted server (a TeXlyre
//      `texlive-ondemand-server` behind a free Cloudflare Tunnel), then
//   2. the public TeXlyre server as a fallback when yours is unreachable.
// The first one that answers at boot is handed to the worker for the session.

import { env } from '$env/dynamic/public';

export type CompileOutcome = { pdf?: string; log?: string; error?: string };

/* eslint-disable @typescript-eslint/no-explicit-any */

const ENGINE_SRC = '/swiftlatex/PdfTeXEngine.js';

/** Public maintained on-demand server — the always-available fallback. */
const TEXLIVE_FALLBACK = 'https://texlive.texlyre.org/';

const withSlash = (u: string) => (u.endsWith('/') ? u : `${u}/`);

/** Ordered TeX Live endpoints: self-hosted (if configured) first, then public. */
const TEXLIVE_ENDPOINTS = [
	...new Set(
		[env.PUBLIC_TEXLIVE_ENDPOINT, TEXLIVE_FALLBACK].filter((u): u is string => !!u).map(withSlash)
	)
];

let endpointPromise: Promise<string> | null = null;

/** Cheap reachability probe — fetch a small known TeX file with a timeout. */
async function reachable(endpoint: string, ms = 6000): Promise<boolean> {
	const ctrl = new AbortController();
	const timer = setTimeout(() => ctrl.abort(), ms);
	try {
		// `pdftex/26/<file>` = kpathsea tex format; article.cls always exists.
		const res = await fetch(`${endpoint}pdftex/26/article.cls`, {
			method: 'GET',
			signal: ctrl.signal,
			cache: 'no-store'
		});
		return res.ok;
	} catch {
		return false;
	} finally {
		clearTimeout(timer);
	}
}

/**
 * Resolve the package server once per session: the first reachable endpoint in
 * priority order. With a single endpoint there's nothing to choose, so we skip
 * the probe; otherwise we fall through to the public server (and let the first
 * real compile surface any remaining error).
 */
function resolveEndpoint(): Promise<string> {
	if (!endpointPromise) {
		endpointPromise = (async () => {
			if (TEXLIVE_ENDPOINTS.length <= 1) return TEXLIVE_ENDPOINTS[0] ?? TEXLIVE_FALLBACK;
			for (const ep of TEXLIVE_ENDPOINTS) {
				if (await reachable(ep)) return ep;
			}
			return TEXLIVE_ENDPOINTS[TEXLIVE_ENDPOINTS.length - 1];
		})();
	}
	return endpointPromise;
}

/**
 * Which package server the engine settled on this session, for a status hint.
 * `self` is true when it's your configured `PUBLIC_TEXLIVE_ENDPOINT` rather than
 * the public fallback. Resolves after the first `getEngine()` / `warmEngine()`.
 */
export async function texliveStatus(): Promise<{ endpoint: string; self: boolean }> {
	const endpoint = await resolveEndpoint();
	return { endpoint, self: endpoint !== TEXLIVE_FALLBACK };
}

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
			// Repoint the package/format server at the first reachable endpoint.
			// Posted straight to the worker — the wrapper's setTexliveEndpoint()
			// detaches the worker reference (upstream bug), so we bypass it.
			const endpoint = await resolveEndpoint();
			engine.latexWorker?.postMessage({ cmd: 'settexliveurl', url: endpoint });
			return engine;
		})().catch((e) => {
			enginePromise = null; // allow retry after a failed boot
			throw e;
		});
	}
	return enginePromise;
}

/**
 * Kick off loading + booting the WASM engine ahead of the first compile (~1.7 MB
 * download + worker spin-up), so the first "Compile" feels instant. Safe to call
 * repeatedly — it reuses the single cached engine and swallows boot errors (the
 * next real compile will surface them).
 */
export function warmEngine(): void {
	if (typeof window === 'undefined') return;
	void getEngine().catch(() => {
		/* a real compile will report the failure */
	});
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

	let engine: any;
	try {
		engine = await getEngine();
	} catch {
		return {
			error:
				'Could not start the in-browser LaTeX engine. Check your connection and reload — ' +
				'the engine (~2 MB) downloads on first use.'
		};
	}

	try {
		engine.writeMemFSFile('main.tex', source);
		engine.setEngineMainFile('main.tex');
		const result = await engine.compileLaTeX();
		const log: string = result?.log ?? '';
		const pdf: Uint8Array | undefined = result?.pdf ? new Uint8Array(result.pdf) : undefined;

		// Best-effort, like the desktop engines: if pdfTeX produced a PDF, show it
		// even when it exited non-zero (recoverable errors). The errors live in the
		// log and surface in the Problems panel.
		if (pdf && pdf.byteLength > 0) {
			return { pdf: bytesToBase64(pdf), log };
		}
		return {
			log,
			error: 'LaTeX compilation failed — no PDF was produced. See the Problems panel.'
		};
	} catch (e) {
		return { error: String(e) };
	}
}
