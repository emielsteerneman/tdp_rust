# Crawlability via Prerendering — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every tdpsearch.com content page ship fully-rendered HTML (content + `<title>`/meta/OG/JSON-LD) in the initial response, so all crawlers (not just Googlebot) can index and preview it.

**Architecture:** Keep the SvelteKit `adapter-static` build but turn on `prerender = true` for content routes (paper pages, homepage, connect pages); dynamic routes (search, suggestions, teams/edit) stay client-side via the SPA `fallback`. Prerendering runs at build time against a **live Axum** (which has the data), emitting static HTML that Axum serves exactly as today. The Axum backend gains a dynamic `/sitemap.xml`; a static `robots.txt` is shipped in the build.

**Tech Stack:** SvelteKit 2 + Svelte 5 (`adapter-static`), Rust/Axum backend, Docker Compose, Cloudflare in front.

## Global Constraints

- Package installs resolve through the internal Nexus. Do **not** pass `--registry`/`--index-url` or edit `.npmrc`/registry config. Use plain `npm ci` / `cargo`.
- `paper_lyt` format is `{league.name()}__{year}__{team_name.name}` (double underscore). Rust: `TDPName::get_paper_lyt()`. Frontend: `${paper.league.name}__${paper.year}__${paper.team_name.name}`.
- Site base URL comes from config `website_url` (`configuration::AppConfig`, `Option<String>`), default `https://tdpsearch.com`.
- Web/API server listens on `:50000`. Build-time API origin: `http://localhost:50000` (local) or `http://web:50000` (compose), passed via env `PUBLIC_API_ORIGIN`.
- **Never commit prerendered artifacts.** Output lives in `frontend/build/` (gitignored via `/build`); also gitignore `/static`.
- API responses are wrapped in `{ data: ... }` (see `frontend/src/lib/api.ts` `fetchApi`).
- **Nexus:** npm/cargo installs resolve through the internal Nexus and the environment is
  already configured for it. Run `npm ci` only at **image-build time** (or on the dev host),
  never inside a fresh runtime container that lacks Nexus config. Do not pass registry flags.
- End every commit message with:
  `Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`

---

## File Structure

**Backend (Rust / `web` crate):**
- `web/src/routes/sitemap.rs` (create) — `/sitemap.xml` handler + pure XML builder.
- `web/src/routes/mod.rs` (modify) — register the sitemap route.
- `web/src/state.rs` (modify) — add `website_url: String` to `AppState`.
- `web/src/main.rs` (modify) — pass `config.website_url` into `AppState::new`.

**Frontend (SvelteKit):**
- `frontend/src/lib/config.ts` (create) — build-vs-runtime API origin.
- `frontend/src/lib/api.ts` (modify) — use the origin from `config.ts`.
- `frontend/src/lib/seo.ts` (create) — helpers to build titles/descriptions/JSON-LD.
- `frontend/src/routes/paper/[id]/+page.server.ts` (create) — `prerender`, `entries()`, data `load`.
- `frontend/src/routes/paper/[id]/+page.ts` (delete) — logic moves to `+page.server.ts` and the component.
- `frontend/src/routes/paper/[id]/+page.svelte` (modify) — `<svelte:head>` + `onMount` open-event.
- `frontend/src/routes/(browse)/+page.ts` (modify) — `prerender = true`.
- `frontend/src/routes/(browse)/+page.svelte` (modify) — `<svelte:head>`.
- `frontend/src/routes/(browse)/search/+page.ts` (modify) — `prerender = false`.
- `frontend/src/routes/connect/api/+page.ts`, `connect/mcp/+page.ts` (create) — `prerender = true`.
- `frontend/src/routes/connect/api/+page.svelte`, `connect/mcp/+page.svelte` (modify) — `<svelte:head>`.
- `frontend/src/routes/suggestions/+page.ts`, `teams/edit/+page.ts` (create) — `prerender = false`.
- `frontend/static/robots.txt` (create).
- `frontend/svelte.config.js` (modify) — prerender error handling.

**Infra / tooling:**
- `.gitignore` (modify, repo root) — add `/static`.
- `scripts/prerender.sh` (create).
- `Makefile` (modify) — `prerender` target; wire into `rebuild-index`.
- `Dockerfile` (modify) — drop image-build frontend step + static COPY.
- `docker-compose.yml` (modify) — one-shot `prerender` service + shared `static` volume.
- `CLAUDE.md`, `README.md` (modify) — reflect prerendering.

---

## Task 1: Backend `/sitemap.xml` route

**Files:**
- Create: `web/src/routes/sitemap.rs`
- Modify: `web/src/state.rs`, `web/src/main.rs:72`, `web/src/routes/mod.rs`
- Test: inline `#[cfg(test)]` in `web/src/routes/sitemap.rs`

**Interfaces:**
- Consumes: `api::list_papers::list_papers(metadata_client, PaperFilter::default(), &dispatcher, EventSource::Web)` → `Vec<TDPName>`; `TDPName::get_paper_lyt()`.
- Produces: `pub async fn sitemap_handler(State<AppState>) -> Response`; `fn build_sitemap_xml(base_url: &str, paper_lyts: &[String]) -> String`; `AppState.website_url: String`.

- [ ] **Step 1: Add `website_url` to `AppState`**

In `web/src/state.rs`, add the field and constructor param:

```rust
#[derive(Clone)]
pub struct AppState {
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub searcher: Arc<Searcher>,
    pub dispatcher: Arc<EventDispatcher>,
    pub tdps_markdown_root: String,
    pub tdps_pdf_root: String,
    pub registry: Option<Arc<dyn RegistryClient + Send + Sync>>,
    pub website_url: String,
}

impl AppState {
    pub fn new(
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        searcher: Arc<Searcher>,
        dispatcher: Arc<EventDispatcher>,
        tdps_markdown_root: String,
        tdps_pdf_root: String,
        registry: Option<Arc<dyn RegistryClient + Send + Sync>>,
        website_url: String,
    ) -> Self {
        Self {
            metadata_client,
            searcher,
            dispatcher,
            tdps_markdown_root,
            tdps_pdf_root,
            registry,
            website_url,
        }
    }
}
```

- [ ] **Step 2: Pass `website_url` at the call site**

In `web/src/main.rs`, the `AppState::new(...)` call (around line 72) — add a final argument after the `registry` arg:

```rust
        config.website_url.clone().unwrap_or_else(|| "https://tdpsearch.com".to_string()),
```

- [ ] **Step 3: Write the failing test for the XML builder**

Create `web/src/routes/sitemap.rs` with only the test + an empty stub:

```rust
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::state::AppState;

fn build_sitemap_xml(base_url: &str, paper_lyts: &[String]) -> String {
    String::new() // stub — replaced in Step 5
}

#[cfg(test)]
mod tests {
    use super::build_sitemap_xml;

    #[test]
    fn builds_urlset_with_static_and_paper_urls() {
        let xml = build_sitemap_xml(
            "https://tdpsearch.com",
            &["soccer_smallsize__2024__RoboTeam_Twente".to_string()],
        );
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"));
        // homepage present
        assert!(xml.contains("<loc>https://tdpsearch.com/</loc>"));
        // paper URL present, correctly joined
        assert!(xml.contains(
            "<loc>https://tdpsearch.com/paper/soccer_smallsize__2024__RoboTeam_Twente</loc>"
        ));
        assert!(xml.trim_end().ends_with("</urlset>"));
    }

    #[test]
    fn trims_trailing_slash_on_base_url() {
        let xml = build_sitemap_xml("https://tdpsearch.com/", &[]);
        assert!(xml.contains("<loc>https://tdpsearch.com/</loc>"));
        assert!(!xml.contains("tdpsearch.com//"));
    }
}
```

- [ ] **Step 4: Run the test to verify it fails**

Run: `cargo test -p web sitemap`
Expected: FAIL (assertions fail — builder returns empty string).

- [ ] **Step 5: Implement the builder + handler**

Replace the stub in `web/src/routes/sitemap.rs`:

```rust
use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::state::AppState;

fn build_sitemap_xml(base_url: &str, paper_lyts: &[String]) -> String {
    let base = base_url.trim_end_matches('/');
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    // Static, prerendered pages.
    for path in ["/", "/connect/mcp", "/connect/api"] {
        out.push_str(&format!("  <url><loc>{base}{path}</loc></url>\n"));
    }
    // One entry per paper.
    for lyt in paper_lyts {
        out.push_str(&format!("  <url><loc>{base}/paper/{lyt}</loc></url>\n"));
    }

    out.push_str("</urlset>\n");
    out
}

pub async fn sitemap_handler(State(state): State<AppState>) -> Response {
    let papers = api::list_papers::list_papers(
        state.metadata_client.clone(),
        api::paper_filter::PaperFilter::default(),
        &state.dispatcher,
        event_processing::EventSource::Web,
    )
    .await;

    let paper_lyts: Vec<String> = match papers {
        Ok(list) => list.iter().map(|p| p.get_paper_lyt()).collect(),
        Err(e) => {
            eprintln!("sitemap: failed to list papers: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let xml = build_sitemap_xml(&state.website_url, &paper_lyts);
    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    )
        .into_response()
}
```

- [ ] **Step 6: Register the route + module**

In `web/src/routes/mod.rs`: add `mod sitemap;` with the other `mod` lines, and register the route on the **`tdps_routes`** router (no activity logging, like the file routes). After the `tdps_routes` block, add the route to it — change:

```rust
    let tdps_routes = Router::new()
        .route("/tdps/{*path}", get(tdps::serve_tdps_file))
        .route("/sitemap.xml", get(sitemap::sitemap_handler))
        .with_state(state.clone());
```

- [ ] **Step 7: Run tests + build to verify it passes**

Run: `cargo test -p web sitemap && cargo build -p web`
Expected: tests PASS, build succeeds.

- [ ] **Step 8: Manual smoke check**

Run (with a local Qdrant + config in place): `make web` in one shell, then:
`curl -s http://localhost:50000/sitemap.xml | head -5`
Expected: XML declaration, `<urlset ...>`, `<loc>…/</loc>`, and paper `<loc>` lines.

- [ ] **Step 9: Commit**

```bash
git add web/src/routes/sitemap.rs web/src/routes/mod.rs web/src/state.rs web/src/main.rs
git commit -m "feat(web): add dynamic /sitemap.xml route

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Task 2: `robots.txt` + gitignore hygiene

**Files:**
- Create: `frontend/static/robots.txt`
- Create: `frontend/static/og-default.png`
- Modify: `.gitignore` (repo root)

**Interfaces:**
- Produces: a `/robots.txt` static asset and a `/og-default.png` (1200×630) link-preview image, both landing in `frontend/build/` and served by Axum's `ServeDir`.

- [ ] **Step 1: Create `frontend/static/robots.txt`**

```
User-agent: *
Allow: /

# AI/answer crawlers — explicitly allowed for citations
User-agent: GPTBot
Allow: /
User-agent: OAI-SearchBot
Allow: /
User-agent: ClaudeBot
Allow: /
User-agent: PerplexityBot
Allow: /
User-agent: CCBot
Allow: /
User-agent: Google-Extended
Allow: /
User-agent: Applebot-Extended
Allow: /
User-agent: Amazonbot
Allow: /

Sitemap: https://tdpsearch.com/sitemap.xml
```

- [ ] **Step 2: Add the default OG image**

Create `frontend/static/og-default.png` — a **1200×630** PNG (site logo/wordmark on a solid
background). If no brand asset exists yet, generate a simple placeholder at the correct
dimensions, e.g.:

```bash
# Requires ImageMagick; any 1200x630 PNG is fine as a starting point.
convert -size 1200x630 xc:'#0f172a' \
  -gravity center -fill white -pointsize 72 -annotate 0 'TDPSearch' \
  frontend/static/og-default.png
```

Referenced as `https://tdpsearch.com/og-default.png` in the OG tags (Tasks 4 & 5).

- [ ] **Step 3: Add `/static` to root `.gitignore`**

Append to `.gitignore` (repo root), under `frontend/.svelte-kit`:

```
/static
```

(`frontend/build` is already ignored via the existing `/build` in `frontend/.gitignore`.
`og-default.png` and `robots.txt` live in `frontend/static/`, which is **source** and IS
tracked — only the generated `frontend/build/` output and the repo-root `/static` symlink
target are ignored.)

- [ ] **Step 4: Verify source files are tracked and build/static are ignored**

Run: `git status --porcelain && git check-ignore -v static 2>/dev/null; echo "---"; git check-ignore -v frontend/build 2>/dev/null`
Expected: `frontend/static/robots.txt` and `frontend/static/og-default.png` show as new tracked files; the repo-root `static` and `frontend/build` are reported as ignored.

- [ ] **Step 5: Commit**

```bash
git add frontend/static/robots.txt frontend/static/og-default.png .gitignore
git commit -m "feat(frontend): add robots.txt + default OG image; gitignore /static

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Task 3: Build-vs-runtime API origin

**Files:**
- Create: `frontend/src/lib/config.ts`
- Modify: `frontend/src/lib/api.ts:9`
- Modify: `frontend/src/routes/paper/[id]/+page.ts` (temporarily; deleted in Task 4 — the `/tdps` fetch there must use the origin during prerender, but Task 4 supersedes this file. Apply the origin in Task 4's new `+page.server.ts` instead; no change here.)

**Interfaces:**
- Produces: `export const API_ORIGIN: string` — `PUBLIC_API_ORIGIN` when `building`, else `''` (relative, same-origin at runtime).

- [ ] **Step 1: Create `frontend/src/lib/config.ts`**

```ts
import { building } from '$app/environment';
import { env } from '$env/dynamic/public';

// At build time (prerendering) we must call a live Axum by absolute URL,
// because /api and /tdps are not SvelteKit routes. At runtime in the
// browser we use same-origin relative paths (Axum serves the static build).
export const API_ORIGIN: string = building ? (env.PUBLIC_API_ORIGIN ?? '') : '';
```

- [ ] **Step 2: Use it in `api.ts`**

In `frontend/src/lib/api.ts`, replace `const API_BASE = '/api';` with:

```ts
import { API_ORIGIN } from './config';

const API_BASE = `${API_ORIGIN}/api`;
```

- [ ] **Step 3: Type-check**

Run: `cd frontend && npm run check`
Expected: no new errors from `config.ts`/`api.ts`. (Pre-existing warnings unrelated to this change are acceptable.)

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/config.ts frontend/src/lib/api.ts
git commit -m "feat(frontend): API origin switches to absolute URL during prerender

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Task 4: Prerender the paper route (server load + entries + head + analytics)

**Files:**
- Create: `frontend/src/lib/seo.ts`
- Create: `frontend/src/routes/paper/[id]/+page.server.ts`
- Delete: `frontend/src/routes/paper/[id]/+page.ts`
- Modify: `frontend/src/routes/paper/[id]/+page.svelte`

**Interfaces:**
- Consumes: `API_ORIGIN` (Task 3); `getTeamInfo`, `listPapers` from `$lib/api`; `RegistryEntry`, `TDPName` types.
- Produces: page `data` shape `{ rawMarkdown, paper_lyt, teamEntries, teamPrettyName, leagueMachine, year }` (unchanged from the old `+page.ts`), plus `title`, `description` for the head.

- [ ] **Step 1: Create `frontend/src/lib/seo.ts`**

```ts
const PRETTY_LEAGUE: Record<string, string> = {
	soccer_smallsize: 'Soccer SmallSize',
	soccer_midsize: 'Soccer MiddleSize'
	// Non-exhaustive: unknown machine names fall back to a title-cased form below.
};

export function prettyLeague(machine: string): string {
	return (
		PRETTY_LEAGUE[machine] ??
		machine
			.split('_')
			.map((w) => w.charAt(0).toUpperCase() + w.slice(1))
			.join(' ')
	);
}

export function paperTitle(team: string, leagueMachine: string, year: number): string {
	return `${team} — ${prettyLeague(leagueMachine)} ${year} | TDPSearch`;
}

/** First ~155 chars of the markdown body, stripped of markup, for meta description. */
export function metaDescription(markdown: string, team: string, year: number): string {
	const text = markdown
		.replace(/^#.*$/gm, '') // drop heading lines
		.replace(/[#*_`>\[\]()!]/g, '') // drop common md punctuation
		.replace(/\s+/g, ' ')
		.trim();
	const fallback = `Team Description Paper by ${team} (${year}), on tdpsearch.com.`;
	const base = text.length > 0 ? text : fallback;
	return base.length > 155 ? base.slice(0, 152).trimEnd() + '…' : base;
}

export function articleJsonLd(opts: {
	title: string;
	description: string;
	url: string;
	team: string;
	year: number;
	league: string;
}): string {
	return JSON.stringify({
		'@context': 'https://schema.org',
		'@type': 'ScholarlyArticle',
		headline: opts.title,
		description: opts.description,
		url: opts.url,
		datePublished: String(opts.year),
		author: { '@type': 'Organization', name: opts.team },
		about: prettyLeague(opts.league),
		isPartOf: { '@type': 'WebSite', name: 'TDPSearch', url: 'https://tdpsearch.com' }
	});
}
```

- [ ] **Step 2: Create `frontend/src/routes/paper/[id]/+page.server.ts`**

```ts
import type { PageServerLoad, EntryGenerator } from './$types';
import { error } from '@sveltejs/kit';
import { API_ORIGIN } from '$lib/config';
import { getTeamInfo, listPapers } from '$lib/api';
import type { RegistryEntry } from '$lib/api';
import { paperTitle, metaDescription } from '$lib/seo';

export const prerender = true;

// Tell the prerenderer every paper page to generate.
export const entries: EntryGenerator = async () => {
	const papers = await listPapers(fetch);
	return papers.map((p) => ({
		id: `${p.league.name}__${p.year}__${p.team_name.name}`
	}));
};

export const load: PageServerLoad = async ({ params, fetch }) => {
	const paper_lyt = params.id;

	const parts = paper_lyt.split('__');
	const leagueMachine = parts[0] ?? '';
	const year = parseInt(parts[1], 10) || 0;
	const teamPrettyName = parts[2]?.replace(/_/g, ' ') ?? '';

	const [rawMarkdown, teamEntries] = await Promise.all([
		fetch(`${API_ORIGIN}/tdps/${paper_lyt}.md`).then((r) => {
			if (!r.ok) throw error(r.status, 'Paper not found');
			return r.text();
		}),
		getTeamInfo(teamPrettyName, fetch).catch((): RegistryEntry[] => [])
	]);

	const title = paperTitle(teamPrettyName, leagueMachine, year);
	const description = metaDescription(rawMarkdown, teamPrettyName, year);

	return { rawMarkdown, paper_lyt, teamEntries, teamPrettyName, leagueMachine, year, title, description };
};
```

- [ ] **Step 3: Delete the old universal load**

Run: `git rm frontend/src/routes/paper/[id]/+page.ts`
(The data fetch moves to `+page.server.ts`; the `/open` POST moves to the component in Step 4. The old file also fired `/open` inside `load` — that is exactly the build-time/undercount bug we are removing.)

- [ ] **Step 4: Add `<svelte:head>` + `onMount` analytics to `+page.svelte`**

At the top of the `<script lang="ts">` block in `frontend/src/routes/paper/[id]/+page.svelte`, add imports and the mount hook (after `let { data }: { data: PageData } = $props();`):

```ts
	import { onMount } from 'svelte';
	import { articleJsonLd } from '$lib/seo';

	// Record a real human view — browser-only, fires on initial hydration and
	// on client-side navigation, never during prerender, never for non-JS bots.
	onMount(() => {
		fetch(`/api/papers/${encodeURIComponent(data.paper_lyt)}/open`, { method: 'POST' }).catch(
			() => {}
		);
	});

	const canonical = $derived(`https://tdpsearch.com/paper/${data.paper_lyt}`);
	const jsonLd = $derived(
		articleJsonLd({
			title: data.title,
			description: data.description,
			url: canonical,
			team: data.teamPrettyName,
			year: data.year,
			league: data.leagueMachine
		})
	);
```

Then add a `<svelte:head>` block immediately after the closing `</script>` tag (before the existing markup):

```svelte
<svelte:head>
	<title>{data.title}</title>
	<meta name="description" content={data.description} />
	<link rel="canonical" href={canonical} />
	<meta property="og:title" content={data.title} />
	<meta property="og:description" content={data.description} />
	<meta property="og:url" content={canonical} />
	<meta property="og:type" content="article" />
	<meta property="og:image" content="https://tdpsearch.com/og-default.png" />
	<meta name="twitter:card" content="summary_large_image" />
	{@html `<script type="application/ld+json">${jsonLd}</` + `script>`}
</svelte:head>
```

(The split `</` + `script>` prevents the Svelte compiler from closing the block early.)

- [ ] **Step 5: Type-check**

Run: `cd frontend && npm run check`
Expected: no errors referencing `paper/[id]`.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/lib/seo.ts frontend/src/routes/paper/
git commit -m "feat(frontend): prerender paper pages with head metadata + JSON-LD

Move data load to +page.server.ts with prerender+entries; add
<svelte:head> title/meta/OG/JSON-LD; move the open-event to onMount
so it counts real views only (never at build, never for bots).

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Task 5: Prerender homepage + connect pages; keep dynamic routes CSR

**Files:**
- Modify: `frontend/src/routes/(browse)/+page.ts`, `frontend/src/routes/(browse)/+page.svelte`
- Modify: `frontend/src/routes/(browse)/search/+page.ts`
- Create: `frontend/src/routes/connect/api/+page.ts`, `frontend/src/routes/connect/mcp/+page.ts`
- Modify: `frontend/src/routes/connect/api/+page.svelte`, `frontend/src/routes/connect/mcp/+page.svelte`
- Create: `frontend/src/routes/suggestions/+page.ts`, `frontend/src/routes/teams/edit/+page.ts`

**Interfaces:**
- Consumes: nothing new.
- Produces: `prerender` flags per route (true for content pages, false for dynamic pages).

- [ ] **Step 1: Homepage prerender flag**

Replace the contents of `frontend/src/routes/(browse)/+page.ts` with:

```ts
export const prerender = true;
```

- [ ] **Step 2: Homepage `<svelte:head>`**

Add immediately after the `</script>` in `frontend/src/routes/(browse)/+page.svelte`:

```svelte
<svelte:head>
	<title>TDPSearch — Search 2000+ RoboCup Team Description Papers</title>
	<meta
		name="description"
		content="Search and browse 2000+ RoboCup Team Description Papers across all leagues — SmallSize, MiddleSize, Humanoid, @Home, Rescue and more."
	/>
	<link rel="canonical" href="https://tdpsearch.com/" />
	<meta property="og:title" content="TDPSearch — RoboCup Team Description Papers" />
	<meta
		property="og:description"
		content="Search and browse 2000+ RoboCup Team Description Papers across all leagues."
	/>
	<meta property="og:url" content="https://tdpsearch.com/" />
	<meta property="og:type" content="website" />
	<meta property="og:image" content="https://tdpsearch.com/og-default.png" />
	<meta name="twitter:card" content="summary_large_image" />
</svelte:head>
```

- [ ] **Step 3: Keep search CSR**

Add to the **top** of `frontend/src/routes/(browse)/search/+page.ts` (above the existing imports):

```ts
export const prerender = false;
```

- [ ] **Step 4: Connect pages — prerender flags**

Create `frontend/src/routes/connect/api/+page.ts` and `frontend/src/routes/connect/mcp/+page.ts`, each containing:

```ts
export const prerender = true;
```

- [ ] **Step 5: Connect pages — heads**

Add after `</script>` in `frontend/src/routes/connect/api/+page.svelte`:

```svelte
<svelte:head>
	<title>Connect via REST API | TDPSearch</title>
	<meta name="description" content="Access the tdpsearch.com REST API to query 2000+ RoboCup Team Description Papers programmatically." />
	<link rel="canonical" href="https://tdpsearch.com/connect/api" />
</svelte:head>
```

Add after `</script>` in `frontend/src/routes/connect/mcp/+page.svelte`:

```svelte
<svelte:head>
	<title>Connect via MCP | TDPSearch</title>
	<meta name="description" content="Connect AI assistants to tdpsearch.com over MCP to search 2000+ RoboCup Team Description Papers." />
	<link rel="canonical" href="https://tdpsearch.com/connect/mcp" />
</svelte:head>
```

- [ ] **Step 6: Keep suggestions + teams/edit CSR**

Create `frontend/src/routes/suggestions/+page.ts` and `frontend/src/routes/teams/edit/+page.ts`, each containing:

```ts
export const prerender = false;
```

- [ ] **Step 7: Type-check**

Run: `cd frontend && npm run check`
Expected: no new errors.

- [ ] **Step 8: Commit**

```bash
git add frontend/src/routes
git commit -m "feat(frontend): prerender homepage + connect pages; pin dynamic routes to CSR

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Task 6: svelte.config prerender resilience + first real build

**Files:**
- Modify: `frontend/svelte.config.js`

**Interfaces:**
- Consumes: everything above.
- Produces: a build that emits populated HTML into `frontend/build/`.

- [ ] **Step 1: Add prerender error handling**

Replace `frontend/svelte.config.js` with:

```js
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		adapter: adapter({
			fallback: 'index.html' // SPA fallback for non-prerendered (dynamic) routes
		}),
		prerender: {
			// A single missing/broken paper must not fail the whole corpus build.
			handleHttpError: 'warn',
			handleMissingId: 'warn'
		}
	}
};

export default config;
```

- [ ] **Step 2: Build against a live Axum**

In one shell: `make web` (needs local Qdrant + `config.toml`; see repo CLAUDE.md).
In another: `cd frontend && PUBLIC_API_ORIGIN=http://localhost:50000 npm run build`
Expected: build completes; prerendered files appear under `frontend/build/paper/`.

- [ ] **Step 3: Verify prerendered HTML actually contains content + meta**

Run (pick any real paper id from `curl -s http://localhost:50000/api/papers | head`):
```bash
f=$(ls frontend/build/paper/*.html | head -1); echo "$f"
grep -o "<title>[^<]*</title>" "$f"
grep -c "og:title" "$f"
grep -c "application/ld+json" "$f"
```
Expected: a real `<title>` (not empty), `og:title` present (>=1), JSON-LD present (>=1), and the file is non-trivial in size (`wc -c "$f"` well over the ~2 KB empty shell).

- [ ] **Step 4: Verify the empty-shell fallback still exists for dynamic routes**

Run: `test -f frontend/build/index.html && echo "fallback present"`
Expected: `fallback present` (SPA fallback for /search etc.).

- [ ] **Step 5: Commit**

```bash
git add frontend/svelte.config.js
git commit -m "feat(frontend): resilient prerender config (warn on http/id errors)

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Task 7: Build tooling — prerender script, Makefile, Docker, docs

**Files:**
- Create: `scripts/prerender.sh`
- Modify: `Makefile`, `Dockerfile`, `docker-compose.yml`, `scripts/rebuild_index.sh`, `CLAUDE.md`, `README.md`

**Interfaces:**
- Consumes: the working build from Task 6.
- Produces: a repeatable prerender that populates the static dir Axum serves.

- [ ] **Step 1: Create `scripts/prerender.sh`**

```sh
#!/usr/bin/env sh
# Prerender the SvelteKit frontend against a live Axum, then place the output
# where the web server serves it. Requires the API to be reachable.
#   API_ORIGIN   default http://localhost:50000
#   OUT_DIR      default (none) — if set, rsync build output there
set -eu

API_ORIGIN="${API_ORIGIN:-http://localhost:50000}"

echo "prerender: waiting for API at ${API_ORIGIN}/api/papers ..."
i=0
until curl -sf "${API_ORIGIN}/api/papers" >/dev/null 2>&1; do
  i=$((i + 1))
  [ "$i" -gt 60 ] && { echo "prerender: API not reachable after 60s" >&2; exit 1; }
  sleep 2
done

echo "prerender: building frontend ..."
cd "$(dirname "$0")/../frontend"
# Deps are installed at image-build/host time (via Nexus). Only install if absent
# — never repoint the registry.
[ -d node_modules ] || npm ci
PUBLIC_API_ORIGIN="${API_ORIGIN}" npm run build

if [ -n "${OUT_DIR:-}" ]; then
  echo "prerender: syncing build -> ${OUT_DIR}"
  mkdir -p "${OUT_DIR}"
  rm -rf "${OUT_DIR:?}/"* 2>/dev/null || true
  cp -r build/. "${OUT_DIR}/"
fi
echo "prerender: done"
```

Then: `chmod +x scripts/prerender.sh`

- [ ] **Step 2: Add the Makefile target + wire into rebuild-index**

In `Makefile`, add `prerender` to the `.PHONY` line that contains `rebuild-index`, and add the target near the frontend targets:

```make
prerender:
	API_ORIGIN=$(or $(API_ORIGIN),http://localhost:50000) ./scripts/prerender.sh
```

Update the `rebuild-index` target to run prerender after the index rebuild:

```make
rebuild-index:
	./scripts/rebuild_index.sh
	$(MAKE) prerender
```

- [ ] **Step 3: Update the Dockerfile — install deps at build, prerender at runtime**

The frontend deps must be installed where Nexus is configured (image build), but the build
itself needs live data (runtime). So: install deps in the builder, and add a lean
`frontend-prerender` stage that carries the deps + source for the runtime one-shot job.

In `Dockerfile`, change the frontend line (line 44) to install deps only (no build — no data
available at image-build):
```dockerfile
RUN cd frontend && npm ci
```
**Remove** the static copy in the `web` stage (line 85):
```
COPY --from=builder /build/frontend/build /app/static
```
Add an empty static dir in the `web` runtime stage before `CMD` so `ServeDir` has a root:
```dockerfile
RUN mkdir -p /app/static
```
Add a new stage at the end of the Dockerfile (deps + source baked in; no runtime npm install):
```dockerfile
# ============================================================================
# Stage: Frontend prerender (one-shot; run against a live web API)
# ============================================================================
FROM node:22-slim AS frontend-prerender
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl rsync ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
# node_modules already installed via Nexus in the builder stage.
COPY --from=builder /build/frontend /app/frontend
COPY --from=builder /build/scripts /app/scripts
CMD ["sh", "scripts/prerender.sh"]
```
(`apt-get` uses Debian mirrors, not a language package registry — unaffected by the Nexus
policy. Do not remove the Rust build.)

- [ ] **Step 4: Update docker-compose — shared volume + one-shot prerender service**

In `docker-compose.yml`, add a named volume and mount it into `web`, then add the `prerender` service. Under the `web` service add a `volumes:` entry and a healthcheck:

```yaml
    volumes:
      - ./data/activity_docker:/app/activity_data
      - ${TDP_MARKDOWN_ROOT:?Set TDP_MARKDOWN_ROOT in .env}:/data/tdps_markdown:ro
      - ${TDP_PDF_ROOT:?Set TDP_PDF_ROOT in .env}:/data/tdps_pdf:ro
      - web_static:/app/static
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:50000/api/papers"]
      interval: 5s
      timeout: 3s
      retries: 20
```

Add a new service (Node image builds the frontend into the shared volume) after `web`:

```yaml
  # One-shot: prerender the frontend against the live web API into the shared
  # static volume. Exits when done. Re-run with: docker compose up prerender
  prerender:
    build:
      context: .
      dockerfile: Dockerfile
      target: frontend-prerender
    container_name: tdp-prerender
    depends_on:
      web:
        condition: service_healthy
    volumes:
      - web_static:/out
    environment:
      - API_ORIGIN=http://web:50000
      - OUT_DIR=/out
    mem_limit: 2g
    restart: "no"
```

And add the volume at the bottom of the file:

```yaml
volumes:
  web_static:
```

(Deps and source are baked into the `frontend-prerender` image at build time, so the job does
no runtime `npm ci` — Nexus-safe. It only reads live data from the web API and writes the
prerendered output into the shared `web_static` volume.)

- [ ] **Step 5: Update `scripts/rebuild_index.sh`**

At the end of `scripts/rebuild_index.sh`, after the docker rebuild step, append:

```sh
echo "rebuild: prerendering frontend"
docker compose up --build prerender
```

- [ ] **Step 6: Verify the full Docker flow**

Run: `docker compose up --build -d qdrant web && docker compose up --build prerender`
Then: `curl -s http://localhost:50000/paper/$(curl -s http://localhost:50000/api/papers | grep -o '"name":"[^"]*"' | head -1) -o /tmp/p.html; grep -o "<title>[^<]*</title>" /tmp/p.html`
Expected: a real `<title>` served by Axum (content came from the shared volume). Also `curl -s http://localhost:50000/robots.txt` returns the robots file and `curl -s http://localhost:50000/sitemap.xml` returns XML.

- [ ] **Step 7: Update docs**

In `CLAUDE.md`: change the `frontend/` crate-overview line from "SvelteKit static SPA" to "SvelteKit app: content routes prerendered at build, dynamic routes (search, forms) client-side via SPA fallback." In the Build & Run section, note that `npm run build` now requires a reachable API (`PUBLIC_API_ORIGIN`) and add `make prerender`. Update the "Static files" prerequisite to mention the prerender step.

In `README.md`: update any build/deploy instructions that assume a data-free frontend build to reference `make prerender` / the compose `prerender` service.

- [ ] **Step 8: Commit**

```bash
git add scripts/prerender.sh scripts/rebuild_index.sh Makefile Dockerfile docker-compose.yml CLAUDE.md README.md
git commit -m "build: prerender via one-shot job against live Axum

Add scripts/prerender.sh + make prerender; docker-compose one-shot
prerender service writing to a shared static volume; drop the data-free
frontend build from the image. Wire prerender into rebuild-index.

Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>"
```

---

## Task 8: End-to-end verification

**Files:** none (verification only).

- [ ] **Step 1: Crawler's-eye content check**

Run: `curl -s https://tdpsearch.com/paper/soccer_smallsize__2024__RoboTeam_Twente | grep -oE "<title>[^<]*</title>|og:title|application/ld\+json"`
Expected: real title + OG + JSON-LD present in raw HTML (no JS executed).

- [ ] **Step 2: Sitemap + robots**

Run: `curl -s https://tdpsearch.com/robots.txt | tail -1; curl -s https://tdpsearch.com/sitemap.xml | grep -c "<loc>"`
Expected: `Sitemap:` line; `<loc>` count ≈ paper count + 3.

- [ ] **Step 3: Analytics correctness**

Confirm in the activity DB (`make activity`) that a `docker compose up prerender` run created **zero** `PaperOpen` events, and that loading a paper page in a real browser creates exactly one. A `curl` of a paper page (no JS) creates none.

- [ ] **Step 4: SPA still works**

In a browser: navigate from the homepage to a paper (client-side, no full reload), use search and filters, and confirm the TOC scroll-spy highlights sections. All must work after hydration.

- [ ] **Step 5: Cloudflare caching**

Run twice: `curl -sI https://tdpsearch.com/paper/soccer_smallsize__2024__RoboTeam_Twente | grep -i cf-cache-status`
Expected: `HIT` on the second request (after setting a `Cache-Control` page rule / header — configure in Cloudflare if not already; this is an ops step, not code).

- [ ] **Step 6: Merge**

Open a PR from `seo/ssr-crawlability` (or fast-forward per your workflow). Confirm CI/build is green before merge.
