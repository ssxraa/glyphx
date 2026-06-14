// Google Analytics (GA4 / gtag.js) for the marketing site only. This is the
// public web app, not the local-first editor — no app document data is ever
// touched here. Everything no-ops until PUBLIC_GA_ID is set, so the site runs
// fine without analytics configured.
//
// Set the measurement ID via the PUBLIC_GA_ID env var (see .env.example),
// e.g. PUBLIC_GA_ID=G-XXXXXXXXXX.

import { browser } from '$app/environment';
import { env } from '$env/dynamic/public';

export const GA_ID = env.PUBLIC_GA_ID ?? '';
export const analyticsEnabled = Boolean(GA_ID);

declare global {
	interface Window {
		dataLayer?: unknown[];
		gtag?: (...args: unknown[]) => void;
	}
}

let loaded = false;

/** Inject gtag.js and configure the property. Safe to call more than once. */
export function loadAnalytics() {
	if (!browser || !analyticsEnabled || loaded) return;
	loaded = true;

	const script = document.createElement('script');
	script.async = true;
	script.src = `https://www.googletagmanager.com/gtag/js?id=${GA_ID}`;
	document.head.appendChild(script);

	window.dataLayer = window.dataLayer ?? [];
	window.gtag = function gtag(...args: unknown[]) {
		window.dataLayer?.push(args);
	};
	window.gtag('js', new Date());
	// Initial page_view is sent automatically here; client-side navigations are
	// reported manually via trackPageview (see +layout.svelte).
	window.gtag('config', GA_ID);
}

/** Report a SPA navigation as a page_view. */
export function trackPageview(path: string, title?: string) {
	if (!browser || !analyticsEnabled || !window.gtag) return;
	window.gtag('event', 'page_view', {
		page_path: path,
		page_location: window.location.href,
		page_title: title ?? document.title
	});
}

/** Report a custom event (download clicks, CTA clicks, etc.). */
export function trackEvent(name: string, params: Record<string, unknown> = {}) {
	if (!browser || !analyticsEnabled || !window.gtag) return;
	window.gtag('event', name, params);
}
