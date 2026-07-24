## Purpose
SvelteKit 2 (Svelte 5) web UI for browsing and searching TDPs. Content routes
(`/`, `/paper/[id]`, `/connect/*`) are prerendered to static HTML at build time (for SEO);
dynamic routes (`/search`, `/suggestions`, `/teams/edit`) run client-side via the SPA
fallback. All served as static files by the `web` (Axum) backend.

## Stack
- Svelte 5 with runes (`$state`, `$derived`, `$effect`)
- Tailwind CSS v4 (via @tailwindcss/vite)
- TypeScript strict mode
- `marked` v12 for markdown rendering

## Key Files
- `src/lib/api.ts` — all backend API calls (fetch wrapper with `/api/*` base).
- `src/lib/types.ts` — TypeScript interfaces mirroring Rust backend types.
- `src/lib/markdown.ts` — custom TDP markdown parser → standard markdown → HTML.
- `src/routes/+layout.ts` — preloads papers, teams, leagues, years on app init.
- `src/routes/(browse)/` — route group for papers listing + search (shares filter sidebar).

## Pages
- `/` — browse papers, filterable by league/year/team. Grouped by year then league.
- `/search?q=...` — search results grouped by paper with chunk scores and breadcrumbs.
- `/paper/[id]` — paper detail with rendered markdown, TOC sidebar, team info sidebar.
- `/connect/mcp` — MCP server setup guides for AI clients.
- `/connect/api` — REST API guide with Python/curl examples, filter values, endpoint listing.
- `/teams/edit` — team metadata editor (requires team auth code).
- `/suggestions` — feedback form.

## Dev Setup
- `npm run dev` proxies `/api/*` and `/tdps/*` to `http://localhost:50000`
- `npm run build` prerenders content routes, so it needs a reachable API: run against a live
  `web` via `PUBLIC_API_ORIGIN` (e.g. `PUBLIC_API_ORIGIN=http://localhost:50000 npm run build`),
  or use `make prerender` / the compose `prerender` service. Output → `frontend/build/`; symlink
  `frontend/build` → `static` for the web server locally.

## Patterns
- Filters are stored in URL search params (shareable links).
- Dark mode: `.dark` class on `<html>`, stored in localStorage, respects system preference.
- Responsive: mobile-first with filter FAB drawer, desktop gets sticky sidebars.
