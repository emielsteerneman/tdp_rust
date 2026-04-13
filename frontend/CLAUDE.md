## Purpose
SvelteKit 2 static SPA (Svelte 5) — the web UI for browsing and searching TDPs.

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
- `npm run build` outputs static files; symlink `frontend/build` → `static` for the web server.

## Patterns
- Filters are stored in URL search params (shareable links).
- Dark mode: `.dark` class on `<html>`, stored in localStorage, respects system preference.
- Responsive: mobile-first with filter FAB drawer, desktop gets sticky sidebars.
