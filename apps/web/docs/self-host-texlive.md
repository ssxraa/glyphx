# Self-hosting the TeX Live package server (web compile)

GlyphX's **web** build compiles LaTeX in the browser with the SwiftLaTeX pdfTeX
WASM engine. The engine itself is self-hosted (`apps/web/static/swiftlatex/`),
but it fetches the format file + every package **on demand** from a TeX Live
"on-demand" server.

The original SwiftLaTeX servers are offline, so the app uses an ordered list:

1. **`PUBLIC_TEXLIVE_ENDPOINT`** — your own server (this guide), tried first.
2. **`https://texlive.texlyre.org/`** — public maintained fallback (third-party,
   slower, no uptime guarantee).

The first endpoint that responds at engine boot is used for the session. So
hosting your own makes web compile fast and self-reliant, while the public
server keeps things working if yours is ever down.

## Why self-host

- **Free.** A Cloudflare Tunnel costs nothing; you only need a machine that's on
  when you want your endpoint serving (your dev box, a homelab box, a small VPS).
  When it's off, the app falls back to the public server automatically.
- **Reliable + fast.** Cloudflare caches responses at the edge; you're not at the
  mercy of a shared public server.
- **Private.** Package requests hit your own infra.

## Setup (Docker + Cloudflare Tunnel)

Uses [TeXlyre/texlive-ondemand-server](https://github.com/TeXlyre/texlive-ondemand-server)
(an adaptation of SwiftLaTeX's server made deployable with Cloudflare Tunnel).

1. On the host machine (Docker + a Cloudflare account with `nexonauts.com` as a
   zone):

   ```sh
   git clone https://github.com/TeXlyre/texlive-ondemand-server
   cd texlive-ondemand-server
   cp envfile .env   # then edit: set HOST_DOMAIN, PORT, CLOUDFLARE_API_KEY
   ```

2. Pick the subdomain for the tunnel, e.g. `texlive.glyphx.nexonauts.com`, and
   run the Cloudflare-tunnel compose:

   ```sh
   chmod +x ./scripts/run_texlive_cloudflare_tunnel.sh
   source ./.env && ./scripts/run_texlive_cloudflare_tunnel.sh "$CLOUDFLARE_API_KEY" "$HOST_DOMAIN" "$PORT"
   ```

   This builds the image, starts the server, and provisions the tunnel +
   DNS/cert for your subdomain.

3. Verify it serves the format file (≈10 MB) and a package, with CORS:

   ```sh
   curl -I https://texlive.glyphx.nexonauts.com/pdftex/0/swiftlatexpdftex.fmt
   curl -I https://texlive.glyphx.nexonauts.com/pdftex/26/article.cls
   # expect: 200, and Access-Control-Allow-Origin in the headers
   ```

4. Point GlyphX at it — set in the deploy host (Cloudflare Worker var) or `.env`:

   ```sh
   PUBLIC_TEXLIVE_ENDPOINT=https://texlive.glyphx.nexonauts.com/
   ```

That's it. Leave it unset to use the public fallback only.

## Notes

- The request protocol is `GET <endpoint>pdftex/<kpathsea-format>/<filename>`
  (and `pdftex/pk/<dpi>/<filename>` for bitmap fonts). The server must send CORS
  headers (TeXlyre's already does via `flask-cors`).
- The on-demand server resolves files from a local TeX Live install via
  kpathsea, so the host needs TeX Live available (the image handles this).
- Engine JS + WASM stay self-hosted in this repo — a CDN can't be the source of
  truth (the official SwiftLaTeX repo ships no built engine, and cross-origin
  Web Workers are blocked by the browser), and self-hosting keeps them working
  offline via the service worker.
