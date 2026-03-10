# Paper Reader Redesign — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the markdown-blob paper view with a structured reader that serves content from the filesystem, renders images inline, and provides a TOC sidebar for navigation.

**Architecture:** The web server mounts `tdps_markdown_root` behind a `/tdps/{lyti}` route that resolves the lyti to the filesystem path. The frontend fetches a single `.md` file, pre-processes the custom markdown format (image blocks) into standard markdown, extracts headings for a TOC sidebar, and renders with `marked`.

**Tech Stack:** Rust/Axum (static file serving), SvelteKit 5 (Svelte 5 runes), marked, Tailwind CSS

---

### Task 1: Backend — Add `/tdps/{lyti}` static file route

**Files:**
- Modify: `web/src/routes/mod.rs`
- Modify: `web/src/state.rs`
- Modify: `web/src/main.rs`

The lyti `soccer_humanoid_adult__2019__NimbRo__0` maps to filesystem path `soccer/humanoid/adult/2019/soccer_humanoid_adult__2019__NimbRo__0`. The league part uses `_` separators that map to `/` directories, but since leagues can be 2 or 3 parts (e.g. `soccer_smallsize` or `soccer_humanoid_adult`), we parse the lyti into a `TDPName` and reconstruct the path from its fields.

**Step 1: Add `tdps_markdown_root` to AppState**

In `web/src/state.rs`, add a `tdps_markdown_root: String` field:

```rust
pub struct AppState {
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub searcher: Arc<Searcher>,
    pub activity_client: Option<Arc<dyn ActivityClient + Send + Sync>>,
    pub tdps_markdown_root: String,
}
```

Update `AppState::new` to accept and store it.

In `web/src/main.rs`, pass `config.data_processing.tdps_markdown_root.clone()` to `AppState::new`.

**Step 2: Add the tdps route handler**

Create `web/src/routes/tdps.rs`:

```rust
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;

use crate::state::AppState;

/// Resolves a lyti + optional subpath to a filesystem path and serves the file.
///
/// Routes:
///   GET /tdps/:lyti.md          -> serves the markdown file
///   GET /tdps/:lyti/*subpath    -> serves files from the lyti's folder (images)
pub async fn serve_tdps_file(
    State(state): State<AppState>,
    Path(path): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    // Split on first '/' to get lyti and optional subpath
    let (lyti_part, subpath) = match path.find('/') {
        Some(idx) => (&path[..idx], Some(&path[idx + 1..])),
        None => (path.as_str(), None),
    };

    // Strip .md extension if present to get the base lyti
    let lyti = lyti_part.strip_suffix(".md").unwrap_or(lyti_part);

    // Parse lyti to get league/year structure
    let tdp_name = data_structures::file::TDPName::try_from(lyti)
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    // Build filesystem path: {root}/{major}/{minor}[/{sub}]/{year}/{lyti}[.md | /{subpath}]
    let league = &tdp_name.league;
    let mut dir = std::path::PathBuf::from(&state.tdps_markdown_root);
    dir.push(&league.league_major);
    dir.push(&league.league_minor);
    if let Some(ref sub) = league.league_sub {
        dir.push(sub);
    }
    dir.push(tdp_name.year.to_string());

    let file_path = match subpath {
        Some(sub) => {
            // Serving a file from the lyti folder (e.g. an image)
            dir.push(lyti);
            dir.push(sub);
            dir
        }
        None => {
            // Serving the markdown file itself
            if lyti_part.ends_with(".md") {
                dir.push(format!("{}.md", lyti));
            } else {
                dir.push(lyti);
            }
            dir
        }
    };

    // Security: ensure resolved path is within tdps_markdown_root
    let canonical_root = std::fs::canonicalize(&state.tdps_markdown_root)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let canonical_file = std::fs::canonicalize(&file_path)
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if !canonical_file.starts_with(&canonical_root) {
        return Err(StatusCode::FORBIDDEN);
    }

    let body = tokio::fs::read(&canonical_file)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Determine content type from extension
    let content_type = match canonical_file.extension().and_then(|e| e.to_str()) {
        Some("md") => "text/markdown; charset=utf-8",
        Some("jpeg") | Some("jpg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };

    Ok(([(header::CONTENT_TYPE, content_type)], body))
}
```

**Step 3: Register the route**

In `web/src/routes/mod.rs`, add `mod tdps;` and register:

```rust
.route("/tdps/{*path}", get(tdps::serve_tdps_file))
```

Add this to the `api_routes` router, before the static files fallback.

**Step 4: Test manually**

Run: `cargo build -p web`
Expected: compiles successfully

**Step 5: Commit**

```bash
git add web/src/routes/tdps.rs web/src/routes/mod.rs web/src/state.rs web/src/main.rs
git commit -m "feat: add /tdps/{lyti} route to serve markdown and images from filesystem"
```

---

### Task 2: Frontend — Markdown pre-processor

**Files:**
- Create: `frontend/src/lib/markdown.ts`

This module transforms the custom TDP markdown format into standard markdown before rendering. It handles:
1. Rewriting `### image` / `#### image_caption` / `#### image_name` blocks into `![caption](/tdps/{lyti}/image_name)`
2. Stripping structural markers (`# paragraph`, `## paragraph_title`, `## paragraph_depth`, `## paragraph_text`, `## images`, `# table`, `# abstract`, `# title`, `# authors`, `# institutions`, `# mailboxes`, `# urls`, `# references`)
3. Extracting headings for the TOC

**Step 1: Create the markdown pre-processor**

```typescript
export interface TocHeading {
  id: string;
  text: string;
  level: number;
}

/**
 * Pre-process custom TDP markdown into standard markdown.
 * Rewrites image blocks and strips structural markers.
 */
export function preprocessMarkdown(raw: string, lyti: string): string {
  const lines = raw.split('\n');
  const output: string[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];
    const trimmed = line.trim();

    // Skip front-matter sections: title, authors, institutions, mailboxes, urls, abstract
    if (/^# (title|authors|institutions|mailboxes|urls|abstract|references)$/i.test(trimmed)) {
      i++;
      // Skip until next top-level heading
      while (i < lines.length && !lines[i].match(/^# /)) {
        i++;
      }
      continue;
    }

    // Skip structural markers
    if (/^## (paragraph_title|paragraph_depth|paragraph_text|images)$/i.test(trimmed)) {
      i++;
      continue;
    }

    if (/^# (paragraph|table)$/i.test(trimmed)) {
      i++;
      continue;
    }

    // Handle image blocks: ### image / #### image_caption / #### image_name
    if (trimmed === '### image') {
      i++;
      let caption = '';
      let imageName = '';

      while (i < lines.length) {
        const imgLine = lines[i].trim();
        if (imgLine === '#### image_caption') {
          i++;
          if (i < lines.length) {
            caption = lines[i].trim();
            i++;
          }
        } else if (imgLine === '#### image_name') {
          i++;
          if (i < lines.length) {
            imageName = lines[i].trim();
            i++;
          }
        } else if (imgLine === '### image' || imgLine.startsWith('# ') || imgLine.startsWith('## ')) {
          break;
        } else {
          i++;
        }
      }

      if (imageName) {
        output.push(`![${caption}](/tdps/${lyti}/${imageName})`);
        output.push('');
      }
      continue;
    }

    output.push(line);
    i++;
  }

  return output.join('\n');
}

/**
 * Extract headings from pre-processed markdown for the TOC sidebar.
 */
export function extractHeadings(markdown: string): TocHeading[] {
  const headings: TocHeading[] = [];
  const lines = markdown.split('\n');

  for (const line of lines) {
    const match = line.match(/^(#{1,3})\s+(.+)$/);
    if (match) {
      const level = match[1].length;
      const text = match[2].trim();
      const id = text
        .toLowerCase()
        .replace(/[^a-z0-9\s-]/g, '')
        .replace(/\s+/g, '-');
      headings.push({ id, text, level });
    }
  }

  return headings;
}
```

**Step 2: Commit**

```bash
git add frontend/src/lib/markdown.ts
git commit -m "feat: add markdown pre-processor for TDP custom format"
```

---

### Task 3: Frontend — TOC sidebar component

**Files:**
- Create: `frontend/src/lib/components/TableOfContents.svelte`

**Step 1: Create the TOC component**

Follows the same pattern as `FilterSidebar.svelte` — sticky sidebar on desktop, collapsible on mobile.

```svelte
<script lang="ts">
  import type { TocHeading } from '$lib/markdown';

  interface Props {
    headings: TocHeading[];
    activeId: string;
  }

  let { headings, activeId }: Props = $props();

  function scrollTo(id: string) {
    const el = document.getElementById(id);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }
</script>

<nav class="hidden lg:block w-64 flex-shrink-0 h-screen sticky top-16 overflow-y-auto border-r border-gray-200 bg-white">
  <div class="p-4">
    <h2 class="text-sm font-semibold text-gray-900 uppercase tracking-wide mb-3">Contents</h2>
    <ul class="space-y-1">
      {#each headings as heading}
        <li>
          <button
            onclick={() => scrollTo(heading.id)}
            class="block w-full text-left text-sm py-1 transition-colors
              {heading.level === 1 ? 'font-medium' : ''}
              {heading.level === 2 ? 'pl-3' : ''}
              {heading.level === 3 ? 'pl-6' : ''}
              {activeId === heading.id
                ? 'text-blue-600 font-medium'
                : 'text-gray-600 hover:text-gray-900'}"
          >
            {heading.text}
          </button>
        </li>
      {/each}
    </ul>
  </div>
</nav>
```

**Step 2: Commit**

```bash
git add frontend/src/lib/components/TableOfContents.svelte
git commit -m "feat: add TableOfContents sidebar component"
```

---

### Task 4: Frontend — Rewrite paper page

**Files:**
- Modify: `frontend/src/routes/paper/[id]/+page.svelte`
- Modify: `frontend/src/routes/paper/[id]/+page.ts` (or `+page.server.ts` if it exists)

**Step 1: Check the current data loading**

The current page loads data via `+page.ts` or `+page.server.ts` which calls the API. We need to change this to fetch from `/tdps/{lyti}.md` instead.

Look at the existing `+page.ts` / `+page.server.ts` in `frontend/src/routes/paper/[id]/` to understand the current data loading pattern, then modify it.

The data loader should:
1. Extract the `id` (lyti) from the route params
2. Fetch `/tdps/{lyti}.md` as text
3. Return the raw markdown + lyti as page data

**Step 2: Rewrite the page component**

```svelte
<script lang="ts">
  import { marked } from 'marked';
  import type { PageData } from './$types';
  import { preprocessMarkdown, extractHeadings } from '$lib/markdown';
  import TableOfContents from '$lib/components/TableOfContents.svelte';

  let { data }: { data: PageData } = $props();

  // Pre-process and render
  const processed = $derived(preprocessMarkdown(data.rawMarkdown, data.lyti));
  const headings = $derived(extractHeadings(processed));
  const htmlContent = $derived(marked.parse(processed) as string);

  // Track active heading on scroll
  let activeId = $state('');

  function handleScroll() {
    const headingElements = headings
      .map(h => document.getElementById(h.id))
      .filter((el): el is HTMLElement => el !== null);

    for (let i = headingElements.length - 1; i >= 0; i--) {
      const rect = headingElements[i].getBoundingClientRect();
      if (rect.top <= 100) {
        activeId = headingElements[i].id;
        return;
      }
    }
    if (headingElements.length > 0) {
      activeId = headingElements[0].id;
    }
  }
</script>

<svelte:window onscroll={handleScroll} />

<div class="flex">
  <TableOfContents {headings} {activeId} />

  <main class="flex-1 min-w-0">
    <div class="max-w-4xl mx-auto px-4 py-6 sm:py-8">
      <article
        class="prose prose-gray prose-sm sm:prose-base max-w-none
          prose-headings:font-bold
          prose-h1:text-2xl sm:prose-h1:text-3xl
          prose-h2:text-xl sm:prose-h2:text-2xl
          prose-h3:text-lg sm:prose-h3:text-xl
          prose-p:leading-relaxed
          prose-a:text-blue-600 hover:prose-a:text-blue-800
          prose-img:rounded-lg prose-img:shadow-md prose-img:mx-auto"
      >
        {@html htmlContent}
      </article>
    </div>
  </main>
</div>
```

**Important:** `marked` needs to generate heading IDs that match our `extractHeadings` IDs. Configure marked's heading renderer or use a slug plugin to ensure IDs match.

**Step 3: Test manually**

Run: `cd frontend && npm run dev`
Navigate to `/paper/soccer_smallsize__2024__RoboTeam_Twente__0`
Expected: paper renders with TOC sidebar, images display inline, tables render as HTML tables

**Step 4: Commit**

```bash
git add frontend/src/routes/paper/[id]/+page.svelte frontend/src/routes/paper/[id]/+page.ts
git commit -m "feat: rewrite paper page with TOC sidebar and filesystem-served content"
```

---

### Task 5: Cleanup — Remove database paper fetch

**Files:**
- Modify: `frontend/src/lib/api.ts` — remove `getPaper` and `getPaperByParams` functions (now unused)
- Verify no other code references these functions

**Step 1: Remove dead code**

Search the frontend for any remaining usage of `getPaper` or `getPaperByParams`. Remove the functions from `api.ts` only if they are no longer called.

Note: keep the `/api/papers/{id}` backend route for now — MCP still uses it.

**Step 2: Commit**

```bash
git add frontend/src/lib/api.ts
git commit -m "chore: remove unused getPaper frontend functions"
```

---

### Task 6: Update types — Sync SearchResultChunk with backend

**Files:**
- Modify: `frontend/src/lib/types.ts`

The current `SearchResultChunk` interface is stale. It has `paragraph_sequence_id`, `chunk_sequence_id`, `idx_begin`, `idx_end` but the backend now sends `content_type`, `content_seq`, `chunk_seq`, `title`. Check the actual backend serialization in `data_structures/src/intermediate/search.rs` and update the TypeScript interface to match.

This is a separate concern from the paper reader but should be done in the same pass to keep types in sync.

**Step 1: Check backend SearchResultChunk fields**

Read `data_structures/src/intermediate/search.rs` to see exact field names.

**Step 2: Update the TypeScript interface**

**Step 3: Commit**

```bash
git add frontend/src/lib/types.ts
git commit -m "fix: sync SearchResultChunk types with backend"
```
