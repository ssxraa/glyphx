# Glyph

Local-first, privacy-first **LaTeX & Typst** editor. A native desktop app
(Tauri) and a free in-browser editor, sharing one UI.

> _"Like Prism, but your documents never leave your machine."_

## Monorepo layout

```
glyph-mvp/
├── apps/
│   ├── web/        SvelteKit · cloud (adapter-auto) · marketing site + /editor · service worker
│   └── desktop/    SvelteKit static SPA (adapter-static) wrapped by Tauri v2 (Rust)
│       └── src-tauri/   Rust shell (crate: glyph)
└── packages/
    └── ui/         @glyph/ui — shared Svelte component library + design system (app.css)
```

- **Package manager:** pnpm workspaces · **Build orchestration:** Turborepo
- **UI:** Svelte 5 (runes) · Tailwind v4 · shadcn-svelte-style components · `tw-animate-css`
- **Design system:** `packages/ui/src/app.css` is the single source of truth for
  tokens — a warm, Clay-derived palette (see `DESIGN.md`). Semantic tokens stay
  calm (editor chrome); the saturated brand tokens drive the marketing surface only.

## Commands

```bash
pnpm install            # install all workspaces

pnpm dev                # run every app in dev
pnpm dev:web            # web only            → http://localhost:5173
pnpm dev:desktop        # desktop (SvelteKit) → http://localhost:1420

pnpm build              # build all apps
pnpm build:web          # web (cloud)
pnpm build:desktop      # desktop frontend → apps/desktop/build (Tauri frontendDist)

pnpm check              # type-check (svelte-check) across workspaces
pnpm format             # prettier --write

# Desktop (Tauri) — from apps/desktop:
pnpm --filter @glyph/desktop desktop:dev     # tauri dev (native window)
pnpm --filter @glyph/desktop desktop:build   # tauri build (native bundle)
```

## Architecture notes

- **Web** keeps SSR (adapter-auto) for SEO-friendly marketing pages; the `/editor`
  route is the in-browser editor. The service worker
  (`apps/web/src/service-worker.ts`) precaches the app shell for offline use.
- **Desktop** is a pure client-side SPA (`ssr = false`) so it runs inside the
  Tauri WebView; the Rust backend will host the in-process compile engines
  (Typst crate today, Tectonic for LaTeX) — not yet wired.
- The `EditorShell` (`@glyph/ui/application`) is the calm, shared editor chrome
  reused by both the desktop window and the web `/editor` route.

## Status

MVP scaffold. Compile engines (Typst WASM in-browser, Typst/Tectonic crates on
desktop), file I/O, and AI features are the next milestones.
