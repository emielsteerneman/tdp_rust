# Design: Make tdpsearch.com crawlable via prerendering

**Date:** 2026-07-20
**Status:** Approved (design), pending implementation plan
**Author:** Emiel + Claude

## Problem

tdpsearch.com ships an **empty client-side-rendered (CSR) shell**. The frontend is
SvelteKit built with `@sveltejs/adapter-static` in pure SPA mode (`fallback: 'index.html'`,
no prerendering), so every URL — homepage and all ~2000 `/paper/{id}` pages — returns the
same HTML with no `<title>`, no meta tags, and no body content. All content is injected
client-side by JavaScript after fetching `/api/*` and `/tdps/{id}.md`.

Consequences (verified by five independent research passes, sources at end):

- **Non-Google search engines** (Bing, DuckDuckGo, Yandex, Baidu, Brave) see a blank page.
- **AI/answer crawlers** (GPTBot, OAI-SearchBot, ClaudeBot, PerplexityBot, CCBot,
  Amazonbot, Bytespider) execute **no JavaScript at all** — they store a blank shell, so
  the corpus is uncitable in AI answers.
- **Social/link-preview unfurlers** (Slack, X, Discord, LinkedIn, Facebook, WhatsApp,
  Telegram, iMessage) read OG tags from the raw `<head>` only — every shared link shows a
  bare URL with no title/description/image.
- **Googlebot** does render JS (~10s median) but with a long delay/failure tail and
  crawl-budget cost; it's the only consumer that copes today.

The universal fix named by every source: **serve real content plus `<title>`/meta/OG in
the initial HTML response, to all user agents.**

## Goal

Every content page (especially `/paper/{id}`) returns fully-rendered HTML — content,
unique `<title>`, meta description, canonical, Open Graph / Twitter tags, and JSON-LD — in
the initial response, with no change to the interactive SPA experience for real users.
Plus a sitemap and robots.txt so crawlers can discover all papers.

## Chosen approach: hybrid prerendering (keep `adapter-static`)

Render the static content routes to **HTML files at build time** (`prerender = true`),
and keep the dynamic routes as client-side SPA via the existing `fallback`. Prerendered
pages still ship the JS bundle and **hydrate** into the full interactive SPA — so search,
filters, the scroll-spy TOC, and instant client-side navigation all keep working. The only
thing prerendering gives up vs request-time SSR is live freshness: **content changes
require a rebuild.**

This fits tdpsearch specifically because TDPs are **immutable once published** and are
**added in deliberate batches** (the year-by-year extraction workflow). A rebuild is one
step appended to an ingestion run — not a daily burden — so we avoid running any extra
production server.

### Approaches considered

| Option | Verdict |
|---|---|
| **Hybrid prerendering (`adapter-static` + `prerender`)** | **Chosen.** No extra production process; Axum keeps serving static files exactly as today. Crawlable populated HTML for the whole corpus; SPA stays fully interactive after hydration. Cost: rebuild on content change (acceptable given batch, immutable content) and the build needs data access. |
| SSR via `adapter-node` | Rejected for operational cost. Always fresh and no rebuilds, but requires a permanent Node process in the request path (SvelteKit SSR needs a JS runtime — Axum can't render Svelte), plus a proxy hook and a Cloudflare-tunnel repoint. Too much standing infrastructure on a shared 10 GiB box for content that rarely changes. |
| Backend HTML injection in Axum | Rejected. Legacy "dynamic rendering" pattern Google advises against; duplicates SvelteKit rendering in Rust (drift); SPA's fresh client mount discards the injected DOM. |

## Architecture

### Production topology — unchanged from today
```
Cloudflare tunnel ──▶ Axum (single process)
                        ├─ /api/*         (data, activity_logging middleware)
                        ├─ /tdps/*.md     (markdown files on disk)
                        ├─ /pdfs/*
                        ├─ /sitemap.xml   (NEW dynamic route — from DB)
                        ├─ /robots.txt    (NEW static file)
                        └─ static/        → prerendered HTML per page
                                            + SPA fallback (index.html)
```
No new container, no proxy, no tunnel change. The `static/` directory simply now contains
**populated** HTML files (one per prerendered route) instead of a single empty shell — plus
`index.html` still present as the SPA fallback for dynamic routes.

### Build step — new
`npm run build` runs the SvelteKit prerenderer (Node, build-time only). It enumerates all
paper IDs and renders each page to a static `.html` + `__data.json`. This requires the
paper data to be reachable during the build (see Component 2). Output is copied into Axum's
`static/` as today.

### What gets prerendered vs stays CSR

| Route | Mode | Why |
|---|---|---|
| `/paper/[id]` | **Prerender** (via `entries()`) | The SEO-critical pages; immutable content |
| `/` (homepage, `(browse)/+page.ts`) | **Prerender** | Landing page; needs meta + content |
| `/connect/api`, `/connect/mcp` | **Prerender** | Static informational pages |
| `/search` | **CSR** (SPA fallback) | Query-driven; not a fixed URL set, no per-query SEO |
| `/suggestions`, `/teams/edit` | **CSR** (SPA fallback) | Interactive/admin; no SEO value |

`prerender = true` is set **per page**, never on the root layout (that would force
un-prerenderable routes like `/search` to prerender).

### Data flow
- **Build:** `entries()` fetches the paper list; each page's server `load` fetches its
  markdown + team info from a running Axum; SvelteKit emits static HTML (content + head) and
  a static `__data.json`.
- **First visit (crawler or human):** Axum serves the prerendered HTML directly —
  populated, crawlable, fast. Human browsers then hydrate into the SPA.
- **Client-side navigation:** SvelteKit loads the target page's static `__data.json` and
  swaps content in-place — no server needed, no reload.

## Components

### 1. Adapter / page options
- Keep `@sveltejs/adapter-static` **with** `fallback: 'index.html'` (SPA fallback stays for
  the CSR routes). No adapter swap.
- Add `export const prerender = true` to the prerendered routes (paper, homepage, connect).
- Ensure SSR is on for those routes at build (default; do not set `ssr = false`).
- Set `trailingSlash` consistently and configure `prerender.handleHttpError` /
  `handleMissingId` so a single bad paper doesn't fail the whole build (log + skip).

### 2. Build-time data access
- The prerender build must reach the paper data. Approach: run the build against a **running
  Axum instance** at a configurable origin (non-secret, e.g. `PUBLIC_API_ORIGIN` such as
  `http://localhost:50000` locally or `http://web:50000` in compose), used only when
  SvelteKit's `building` flag is true. At runtime, loads use the existing relative `/api` and
  `/tdps` paths served by Axum on the same origin.
- Exact env plumbing (guarding the build origin behind `building`, keeping it out of the
  client runtime path) is finalized in the implementation plan.

### 3. `entries()` for `/paper/[id]`
- In `routes/paper/[id]/+page.server.ts`: `export const prerender = true` and
  `export async function entries()` returning `[{ id }]` for every paper (from `/api/papers`).
- Move the paper data `load` (markdown + team registry) into this **server** load so it runs
  only at build and emits a static `__data.json` — no runtime server required for client nav.

### 4. Per-page metadata — `<svelte:head>`
For each prerendered route, return SEO fields from `load` and render in `<svelte:head>`
(baked into the prerendered HTML):
- `<title>` — unique, e.g. `"CAU Mountain Sea — Soccer Humanoid AdultSize 2025 | TDPSearch"`.
- `<meta name="description">` — paper abstract / first paragraph, truncated (~155 chars).
- `<link rel="canonical">` — absolute `https://tdpsearch.com/paper/{id}`.
- Open Graph: `og:title`, `og:description`, `og:url`, `og:type=article`, `og:image`.
- Twitter: `twitter:card=summary_large_image`.
- **JSON-LD** (`ScholarlyArticle` / `TechArticle`) with title, author (team), year, league.
- Tags early in `<head>` (Slack fetches ~32 KB, Facebook cuts at 1 MB).
- `og:image`: one site-wide 1200×630 HTTPS image initially; per-paper optional later.

### 5. `/sitemap.xml` (dynamic, Axum) + `/robots.txt` (static, Axum)
- **`/sitemap.xml`**: new Axum route (`web/src/routes/`) querying the paper list from the DB
  and emitting a `<url>` per `/paper/{id}` plus the main pages. Serving it from Axum keeps it
  **always current** (no rebuild needed) and is trivial in Rust. The rebuild that publishes
  new prerendered pages runs in the same ingestion batch, keeping sitemap and pages aligned.
- **`/robots.txt`**: static file in `static/`. `Allow: /`; do not block `/api` or `/tdps`;
  explicitly allow AI user-agents (GPTBot, OAI-SearchBot, ClaudeBot, PerplexityBot, CCBot,
  Google-Extended, Applebot-Extended, Amazonbot); `Sitemap: https://tdpsearch.com/sitemap.xml`.

### 6. Cloudflare caching
- Prerendered pages are static files — set long-lived `Cache-Control` so Cloudflare caches
  them at the edge. Trivial and safe; does not affect analytics (see Component 7).

### 7. Analytics — move `/open` to `onMount` (required)
- **Current bug under prerendering:** `routes/paper/[id]/+page.ts:24` fires
  `POST /api/papers/{id}/open` inside `load`. Under prerendering that would (a) fire ~2000
  times **at build**, and (b) **not fire on a direct visit** to a prerendered page (its
  `load` data is baked in, so `load` doesn't re-run on initial hydration) — undercounting
  real views.
- **Fix:** move the `/open` call into **`onMount`** in `paper/[id]/+page.svelte`. `onMount`
  runs client-side only, on every page view (initial hydration *and* client-side nav), and
  never at build. This counts every real human view and excludes build and non-JS bots.
- `POST /api/papers/{id}/pdf-open` (`+page.svelte:57`) is already in a click handler
  (browser-only) — unchanged. No third-party analytics exist.
- Because analytics is a client-side POST to `/api` (never cached; POSTs aren't cached),
  Cloudflare caching the static HTML has no effect on stats.

## Build & deployment

**No production topology change.** Axum still serves everything; no Node server, no proxy,
no tunnel repoint.

**The build-time-data constraint (key change).** The current `Dockerfile` builds the
frontend inside the Rust builder stage (`RUN cd frontend && npm ci && npm run build`, line
44), where no backend runs. That works for a pure SPA but **fails under prerendering**,
which fetches paper data. Prerendering must therefore run **against a live Axum**, not
during `docker build`. Chosen mechanism:

- **Docker:** a **one-shot `prerender` compose service** (Node image) that `depends_on` a
  healthy `web`, runs `npm run build` with `PUBLIC_API_ORIGIN=http://web:50000`, and writes
  output into a **shared volume** mounted at web's `/app/static`. It exits when done.
  Re-running it (`docker compose up prerender`) re-prerenders after new papers. The web
  image no longer builds the frontend (remove line 44 and the `COPY .../frontend/build
  /app/static`); web serves whatever is in the shared static volume.
  - Startup ordering: `qdrant → web (API live, static initially empty) → prerender (populates
    volume)`. Brief window where pages 404 until prerender completes; gate with a healthcheck
    if needed.
- **Local dev:** unchanged for day-to-day work (`make ui` = Vite dev server, no prerender).
  To reproduce prod output locally: run `make web`, then a new `make prerender` target that
  runs the build against `http://localhost:50000` into `frontend/build` (symlinked to
  `static` as today).

**Rebuild trigger.** Append a prerender step to the existing ingestion/extraction workflow so
newly imported papers get prerendered pages. `make rebuild-index` (teardown → reindex →
snapshot → docker rebuild) should also trigger the `prerender` job.

**Build cost:** ~2000 pages (fetch markdown + render) is a multi-minute batch job — fine for
an occasional rebuild. Output is ~2000 small HTML + `__data.json` files.

**Axum change:** add `/sitemap.xml` route; add `robots.txt` to `static/`. Keep the existing
`ServeDir` + `index.html` fallback.

## Scripts, Makefile & docs to update
Prerendering changes the build contract, so the surrounding tooling must move in lockstep
(don't leave stale "static SPA" assumptions):

- **`Dockerfile`** — remove the image-build frontend step (line 44) and the
  `COPY --from=builder /build/frontend/build /app/static` (line 85); web serves from the
  shared static volume instead.
- **`docker-compose.yml`** — add the one-shot `prerender` service (Node, `PUBLIC_API_ORIGIN`,
  `depends_on: web` healthy) and a shared `static` volume mounted into both `web` and
  `prerender`. Set a memory limit on the prerender job (shared 10 GiB N150).
- **`Makefile`** — add `prerender` (build against a running Axum) and wire it into
  `rebuild-index`; the `docker` target should bring up `prerender` after `web`. `ui` (Vite
  dev) unchanged.
- **`scripts/`** — add `scripts/prerender.sh` (wait for `/api` health, then run the build with
  the right origin and output path). Update `scripts/rebuild_index.sh` to call it.
- **`web`** — add the `/sitemap.xml` handler + route registration; drop nothing from the
  existing static serving.
- **`CLAUDE.md`** — update the architecture line ("SvelteKit static SPA" → "prerendered pages
  + SPA fallback"), the Build & Run frontend section (`npm run build` now needs a live API),
  and the static-files/symlink note.
- **`README.md`** — update any build/deploy instructions that assume a data-free frontend build.

## Git hygiene — no prerendered artifacts in git
- Prerendered output stays in **`frontend/build/`**, already gitignored via `/build`.
- Add **`/static`** to the root `.gitignore` (local dev symlinks `static → frontend/build`;
  prevent accidental commits of the populated dir).
- Never commit generated HTML/`__data.json`. Source diffs stay small; adding papers produces
  **zero** tracked-file changes — only a rebuild of ignored artifacts.

## Error handling
- Prerender: `handleHttpError`/missing markdown → log and skip that page, don't fail the
  build; surface a summary of skipped papers.
- `load`: markdown 404 → SvelteKit `error(404)` (already the pattern) → a proper 404 page.
- Never emit `noindex` in prerendered HTML (fatal for indexing).

## Testing / verification
- `curl https://tdpsearch.com/paper/{id}` returns HTML containing the real `<title>`, meta
  description, OG tags, JSON-LD, and body text (no JS execution).
- `curl https://tdpsearch.com/sitemap.xml` lists all paper URLs; `/robots.txt` references it.
- Link-preview debuggers (Slack/X/Facebook) show title + description + image.
- Google URL Inspection / Bing inspection see content without rendering.
- Analytics: a build run creates **zero** `open` events; a direct browser visit to a
  prerendered paper creates exactly one; a non-JS bot fetch creates none.
- Confirm client-side navigation between papers still works with no live server (static
  `__data.json` loads) and that search/filters/TOC remain interactive after hydration.
- Cloudflare `cf-cache-status: HIT` on repeat paper fetches.

## Out of scope (for now)
- Per-paper OG images (start with one site-wide 1200×630 image).
- `llms.txt` (no major AI provider consumes it yet).
- Request-time SSR / `adapter-node` (revisit only if content ever needs to be live-fresh
  without a rebuild).

## Research sources
- Google JS SEO Basics; Dynamic Rendering "workaround" note; Cloaking guidelines
  (developers.google.com/search).
- Vercel + MERJ: "How Google Handles JavaScript" and "The rise of the AI crawler."
- Bing Webmaster JS/dynamic-rendering guidance; Screaming Frog Bing JS test.
- SvelteKit docs: SEO, Page options, Adapters, adapter-static/SSG (prerender + `entries`),
  Single-page apps, Glossary.
- Platform OG/Card docs: Facebook, Slack, X/Twitter, Telegram, Apple TN3156.
