/// <reference types="@sveltejs/kit" />
/// <reference no-default-lib="true"/>
/// <reference lib="esnext" />
/// <reference lib="webworker" />

// SvelteKit's built-in service worker. SvelteKit bundles this file and exposes
// `$service-worker` with the precise list of build assets + a content hash.
// This is what powers Glyph's "open the browser, works offline" promise — no
// Workbox, no generated *.sw.js, just the framework primitive.

import { build, files, version } from '$service-worker';

const sw = self as unknown as ServiceWorkerGlobalScope;

const CACHE = `glyph-cache-${version}`;

// App shell + static assets to precache on install.
const PRECACHE = [...build, ...files];

sw.addEventListener('install', (event) => {
	event.waitUntil(
		(async () => {
			const cache = await caches.open(CACHE);
			await cache.addAll(PRECACHE);
			await sw.skipWaiting();
		})()
	);
});

sw.addEventListener('activate', (event) => {
	event.waitUntil(
		(async () => {
			// Drop caches from previous deployments.
			for (const key of await caches.keys()) {
				if (key !== CACHE) await caches.delete(key);
			}
			await sw.clients.claim();
		})()
	);
});

sw.addEventListener('fetch', (event) => {
	const { request } = event;
	if (request.method !== 'GET') return;

	const url = new URL(request.url);
	if (!url.protocol.startsWith('http')) return;

	event.respondWith(
		(async () => {
			const cache = await caches.open(CACHE);

			// Precached build assets are immutable (hashed) — serve cache-first.
			if (PRECACHE.includes(url.pathname)) {
				const cached = await cache.match(url.pathname);
				if (cached) return cached;
			}

			// Everything else: network-first, fall back to cache when offline.
			try {
				const response = await fetch(request);
				const isCacheable =
					response.status === 200 && response.type === 'basic';
				if (isCacheable) cache.put(request, response.clone());
				return response;
			} catch (err) {
				const cached = await cache.match(request);
				if (cached) return cached;
				throw err;
			}
		})()
	);
});
