# Web Search Improvements — Design Spec

## Goal

Make the web search smarter by leveraging API features that already exist but aren't wired up in the frontend: breadcrumbs, content type filtering, and a better default result limit.

## Changes

### 1. Filter out non-text content

Image captions and table content produce low-quality search results in the web UI. Hardcode `content_type_filter: "text"` for all web searches.

**Files:**
- `frontend/src/lib/types.ts` — add `content_type_filter?: string` to `SearchParams`
- `frontend/src/lib/api.ts` — wire `content_type_filter` to the API call (same pattern as `lyti_filter`)
- `frontend/src/routes/search/+page.ts` — pass `content_type_filter: "text"` in the search params

The backend (`api/src/search.rs`) already accepts `content_type_filter` as a query parameter. No backend changes needed.

### 2. Clickable breadcrumbs on each chunk result

Each `SearchResultChunk` already includes `breadcrumbs: BreadcrumbEntry[]` from the API. These show the section hierarchy where the chunk lives (e.g. `Introduction > 1.1 Related Work`). Currently not rendered.

**Rendering:** Small gray breadcrumb path above each chunk's text content, using `>` as separator.

**Clickable:** Each breadcrumb segment links to `/paper/{paperId}#{slugified-title}`, navigating to the paper page and scrolling to that section. The slugify function must match the paper page's heading ID generation:

```
text.toLowerCase().replace(/[^a-z0-9\s-]/g, '').trim().replace(/\s+/g, '-')
```

**Files:**
- `frontend/src/lib/components/ChunkResult.svelte` — accept `breadcrumbs: BreadcrumbEntry[]` and `paperId: string` props; render clickable breadcrumb path above text
- `frontend/src/lib/components/PaperGroup.svelte` — pass `breadcrumbs` and `paperId` (already available as `chunk.breadcrumbs` and `paperId` prop) through to `ChunkResult`

### 3. Bump default result limit to 20

The API defaults to 10 chunks. Increase to 20 for web searches to surface more relevant content without pagination complexity.

**Files:**
- `frontend/src/routes/search/+page.ts` — pass `limit: 20` in the search params

## Non-goals

- No search type selector (hybrid/semantic/keyword) — keep hybrid default
- No UI toggle for content type filtering — hardcoded to text only
- No pagination or "load more" — just a higher fixed limit
- No backend changes
