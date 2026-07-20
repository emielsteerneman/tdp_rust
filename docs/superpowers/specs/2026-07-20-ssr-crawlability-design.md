# Design: Make tdpsearch.com crawlable via SvelteKit SSR

**Date:** 2026-07-20
**Status:** Approved (design), pending implementation plan
**Author:** Emiel + Claude

## Problem

tdpsearch.com ships an **empty client-side-rendered (CSR) shell**. The frontend is
SvelteKit built with `@sveltejs/adapter-static` in SPA mode (`fallback: 'index.html'`),
so every URL — homepage and all ~2000 `/paper/{id}` pages — returns the same HTML with
no `<title>`, no meta tags, and no body content. All content is injected client-side by
JavaScript after fetching `/api/*` and `/tdps/{id}.md`.

Consequences (verified by five independent research passes, sources at end):

- **Non-Google search engines** (Bing, DuckDuckGo, Yandex, Baidu, Brave) see a blank page.
  Bing renders JS only inconsistently and recommends against relying on it; DuckDuckGo
  rides Bing's index.
- **AI/answer crawlers** (GPTBot, OAI-SearchBot, ClaudeBot, PerplexityBot, CCBot,
  Amazonbot, Bytespider) execute **no JavaScript at all** — they store a blank shell,
  so the corpus is uncitable in AI answers.
- **Social/link-preview unfurlers** (Slack, X, Discord, LinkedIn, Facebook, WhatsApp,
  Telegram, iMessage) read OG tags from the raw `<head>` only — every shared link shows a
  bare URL with no title/description/image.
- **Googlebot** does render JS (~10s median) but with a long delay/failure tail and
  crawl-budget cost; it's the only consumer that copes today.

The universal fix named by every source: **serve real content plus `<title>`/meta/OG in
the initial HTML response, to all user agents.**

## Goal

Every page (especially `/paper/{id}`) returns fully-rendered HTML — content, unique
`<title>`, meta description, canonical, Open Graph / Twitter tags, and JSON-LD — in the
initial server response, with no change to the interactive SPA experience for real users.
Plus a sitemap and robots.txt so crawlers can discover all papers.

## Chosen approach: SvelteKit SSR via `adapter-node`

SvelteKit renders each page **on the server** per request, sends complete HTML, then
**hydrates** on the client (attaches listeners without re-rendering). Rendering logic is
written once (in Svelte), and SvelteKit guarantees server/client HTML agree — no drift,
no flash.

### Approaches considered

| Option | Verdict |
|---|---|
| **A. SSR via `adapter-node`** | **Chosen.** SvelteKit's default posture; real HTML + meta with zero hand-written rendering; always fresh from the backend at request time; sidesteps build-time-data problems. |
| B. Prerendering (`prerender = true`) | Rejected as primary. Needs all ~2000 pages' data at frontend-build time (awkward across the Docker split) and goes stale on every new paper — and papers are added continuously. Viable only as a hybrid for a few static pages. |
| C. Backend HTML injection in Axum | Rejected. Not cloaking (same HTML to all UAs), but it's the legacy "dynamic rendering" pattern Google now advises against; duplicates SvelteKit rendering in Rust (drift risk); the SPA's fresh client mount discards the injected DOM (flash + wasted work). |

`adapter-cloudflare` was considered (already behind Cloudflare) but rejected for now:
the SSR worker would have to reach origin Axum through the tunnel — trickier networking
than a Node container co-located with Axum on the Docker network.

## Architecture

### Before
```
Cloudflare tunnel ──▶ Axum (single process)
                        ├─ /api/*      (data, activity_logging middleware)
                        ├─ /tdps/*.md  (markdown files on disk)
                        ├─ /pdfs/*
                        └─ static/     → index.html (empty SPA shell)
```

### After
```
Cloudflare tunnel ──▶ Node SSR server (adapter-node)   ← new public front door
                        │
                        ├─ page routes (/, /paper/[id], /search, /connect/*, …)
                        │     SSR-render HTML, then hydrate on client
                        ├─ /sitemap.xml   (SvelteKit endpoint)
                        ├─ /robots.txt    (SvelteKit endpoint or static)
                        │
                        └─ /api, /tdps, /pdfs ──proxy (handle hook)──▶ Axum (internal)
```

Axum is unchanged except it **stops serving the static frontend** (drops the `ServeDir`
+ SPA fallback). It becomes an internal service reachable only from the Node container.
One public origin, same as today.

### Data flow (paper page)
1. Request `/paper/soccer_..._CAU_Mountain_Sea` hits Node SSR.
2. Route `load` runs **on the server**; `event.fetch('/tdps/{id}.md')` and
   `getTeamInfo(...)` resolve through the `handle` hook → proxied to internal Axum.
3. Node renders full HTML: markdown → body, plus `<svelte:head>` title/meta/OG/JSON-LD.
4. Response sent to client (crawler sees everything). Browser hydrates → SPA behavior.
5. Client-side navigations run the same `load` in the browser, as today.

## Components

### 1. Adapter + SSR config
- `frontend/svelte.config.js`: swap `@sveltejs/adapter-static` → `@sveltejs/adapter-node`;
  remove the SPA `fallback`.
- `frontend/package.json`: replace `adapter-static` dep with `adapter-node`.
- Remove `export const ssr = false` / `prerender = false` where they disable SSR
  (`routes/(browse)/+page.ts` currently pins `prerender = false; ssr = true` — audit all
  route options so SSR is on for content routes).
- Audit `load` functions for SSR-safety: no top-level `window`/`document`. Current loads
  use `fetch` only — safe.

### 2. Proxy hook — `frontend/src/hooks.server.ts` (new)
- `handle`: if `event.url.pathname` starts with `/api`, `/tdps`, or `/pdfs`, forward the
  request (method, headers, body) to the internal Axum base URL and stream the response
  back. Otherwise `resolve(event)` normally.
- Internal Axum base from env var, e.g. `INTERNAL_API_BASE=http://web:8080`.
- Preserve client IP / forwarded headers so Axum `activity_logging` still records the real
  visitor, not the Node process.

### 3. Per-page metadata — `<svelte:head>`
For each content route, return SEO fields from `load` and render in `<svelte:head>`:
- `<title>` — unique, e.g. `"CAU Mountain Sea — Soccer Humanoid AdultSize 2025 | TDPSearch"`.
- `<meta name="description">` — paper abstract or first paragraph, truncated (~155 chars).
- `<link rel="canonical">` — absolute `https://tdpsearch.com/paper/{id}`.
- Open Graph: `og:title`, `og:description`, `og:url`, `og:type=article`, `og:image`.
- Twitter: `twitter:card=summary_large_image`.
- **JSON-LD** (`ScholarlyArticle` / `TechArticle`) with title, author (team), year, league.
- Tags must sit **early in `<head>`** (Slack fetches ~32 KB, Facebook cuts at 1 MB).
- `og:image`: a 1200×630 HTTPS image (site default initially; per-paper optional later).
- Homepage gets site-level title/description/OG.

### 4. `/sitemap.xml` — SvelteKit endpoint (`routes/sitemap.xml/+server.ts`)
- Fetch `/api/papers` (via internal proxy), emit a `<url>` per `/paper/{id}` plus the main
  pages (`/`, `/connect/*`). Always current — no rebuild.
- Set a sensible `Cache-Control` (e.g. a few hours) so it's cheap.

### 5. `/robots.txt` — SvelteKit endpoint or static file
- `Allow: /`; do not block `/api`, `/tdps` resources needed for rendering.
- **Explicitly allow** AI user-agents we want citations from (GPTBot, OAI-SearchBot,
  ClaudeBot, PerplexityBot, CCBot, Google-Extended, Applebot-Extended, Amazonbot).
- `Sitemap: https://tdpsearch.com/sitemap.xml`.

### 6. Cloudflare caching (performance)
- Set `Cache-Control` on paper/browse SSR responses (public, with a revalidation window)
  so Cloudflare caches the rendered HTML at the edge. Most requests — repeat visitors and
  all bots — never hit Node. This neutralizes most SSR per-request cost.
- Keep the homepage/search cache short or dynamic as needed.

### 7. Browser-only analytics guard (required)
- **Gotcha:** `routes/paper/[id]/+page.ts:24` fires `POST /api/papers/{id}/open` inside
  `load`. Under SSR, `load` runs on the server for the initial request, so this would fire
  on **every SSR render including every bot hit**, inflating open counts.
- **Fix:** only fire it in the browser — guard with `if (browser)`
  (`import { browser } from '$app/environment'`) or move into `onMount` in the component.
- `POST /api/papers/{id}/pdf-open` (`+page.svelte:57`) is already in a click handler
  (browser-only) — unchanged. No third-party analytics exist.

## Deployment changes
- **`docker-compose.yml`**: add a `frontend` Node service running `node build/index.js`,
  `depends_on: web`, on the internal network; set `INTERNAL_API_BASE`, `ORIGIN`
  (`https://tdpsearch.com`), and a **memory limit** (N150 is a shared 10 GiB box).
- **`Dockerfile`**: frontend build stage runs `npm run build` (now adapter-node) and the
  runtime stage runs Node instead of copying static files into Axum.
- **Axum**: remove the `ServeDir`/`ServeFile` static-serving from `web/src/routes/mod.rs`
  (Node owns page routes now). `/api`, `/tdps`, `/pdfs` unchanged.
- **Cloudflare tunnel**: repoint the `tdpsearch.com` ingress from Axum's port to the Node
  service's port. (One-line ingress change.)

## Error handling
- Proxy hook: on Axum error/timeout, return the upstream status; log server-side.
- `load`: markdown 404 → SvelteKit `error(404)` renders a proper 404 page (already the
  pattern in `paper/[id]/+page.ts`).
- SSR render failure must not take down navigation — rely on SvelteKit's error boundaries
  (`+error.svelte`).
- Never emit `noindex` in the initial HTML (fatal for indexing even if JS would remove it).

## Testing / verification
- `curl https://tdpsearch.com/paper/{id}` returns HTML containing the real `<title>`,
  meta description, OG tags, and body text (no JS execution).
- `curl https://tdpsearch.com/sitemap.xml` lists all paper URLs; `/robots.txt` references
  the sitemap.
- Link-preview check (Slack/X/Facebook debuggers) shows title + description + image.
- Google Rich Results / URL Inspection renders content; Bing URL inspection sees content.
- Analytics: confirm a bot fetch of a paper page does **not** create an `open` event;
  a real browser visit does.
- Performance: measure TTFB and Node CPU/RSS under load; confirm Cloudflare `cf-cache-status`
  is `HIT` on repeat paper fetches.

## Out of scope (for now)
- Per-paper OG images (start with one site-wide 1200×630 image).
- `llms.txt` (no major AI provider consumes it yet).
- Migrating to `adapter-cloudflare` (revisit if the Node container proves awkward).

## Research sources
- Google JS SEO Basics; Dynamic Rendering "workaround" note; Cloaking guidelines
  (developers.google.com/search).
- Vercel + MERJ: "How Google Handles JavaScript" and "The rise of the AI crawler."
- Bing Webmaster JS/dynamic-rendering guidance; Screaming Frog Bing JS test.
- SvelteKit docs: SEO, Page options, Adapters, adapter-static/SSG, Glossary.
- Platform OG/Card docs: Facebook, Slack, X/Twitter, Telegram, Apple TN3156.
