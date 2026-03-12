# Web Search Improvements Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve web search results by filtering to text-only content, showing clickable breadcrumbs on each result, and increasing the default result limit to 20.

**Architecture:** All changes are in the SvelteKit frontend. The backend already supports `content_type_filter`, breadcrumbs in search results, and configurable limits. We extract the duplicated heading-slug logic into a shared utility, then wire up the missing params and render breadcrumbs.

**Tech Stack:** SvelteKit 5 (runes mode), TypeScript, Tailwind CSS

---

## File Map

| File | Action | Responsibility |
|------|--------|---------------|
| `frontend/src/lib/markdown.ts` | Modify | Export `slugifyHeading()` extracted from existing duplicated logic |
| `frontend/src/lib/types.ts` | Modify | Add `content_type_filter` to `SearchParams` |
| `frontend/src/lib/api.ts` | Modify | Wire `content_type_filter` to API call |
| `frontend/src/routes/search/+page.ts` | Modify | Pass `content_type_filter: "text"` and `limit: 20` |
| `frontend/src/lib/components/ChunkResult.svelte` | Modify | Accept breadcrumb props, render clickable breadcrumb trail |
| `frontend/src/lib/components/PaperGroup.svelte` | Modify | Pass breadcrumb data and paperId through to ChunkResult |
| `frontend/src/routes/paper/[id]/+page.svelte` | Modify | Use shared `slugifyHeading()` instead of inline logic |

---

## Chunk 1: Search params and breadcrumbs

### Task 1: Extract shared slugify utility

The heading ID slug logic is duplicated in `markdown.ts:285-289` and `paper/[id]/+page.svelte:13-17`. Extract it into one shared function.

**Files:**
- Modify: `frontend/src/lib/markdown.ts:285-289`
- Modify: `frontend/src/routes/paper/[id]/+page.svelte:13-17`

- [ ] **Step 1: Add `slugifyHeading` to `markdown.ts`**

Add this exported function before `extractHeadings`:

```typescript
/**
 * Generate a URL-safe heading ID from heading text.
 * Must match the marked renderer's heading ID logic.
 */
export function slugifyHeading(text: string): string {
	return text
		.toLowerCase()
		.replace(/[^a-z0-9\s-]/g, '')
		.trim()
		.replace(/\s+/g, '-');
}
```

Then update `extractHeadings` to use it — replace lines 285-289:

```typescript
const id = slugifyHeading(text);
```

- [ ] **Step 2: Update paper page to use shared slugify**

In `frontend/src/routes/paper/[id]/+page.svelte`, import `slugifyHeading` and replace the inline slug logic in the renderer:

```typescript
import { preprocessMarkdown, extractHeadings, slugifyHeading } from '$lib/markdown';

// Replace renderer.heading (preserve @ts-ignore for marked v12 typing):
// @ts-ignore — marked v12 uses (text, level, raw) not the object form
renderer.heading = (text: string, level: number) => {
    const id = slugifyHeading(text);
    return `<h${level} id="${id}">${text}</h${level}>`;
};
```

- [ ] **Step 3: Verify the frontend builds**

Run: `cd frontend && npm run build`
Expected: Build succeeds with no errors.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/markdown.ts frontend/src/routes/paper/\[id\]/+page.svelte
git commit -m "refactor: extract slugifyHeading into shared utility"
```

---

### Task 2: Wire content_type_filter and limit to search

**Files:**
- Modify: `frontend/src/lib/types.ts:63-71`
- Modify: `frontend/src/lib/api.ts:35-57`
- Modify: `frontend/src/routes/search/+page.ts:19-24`

- [ ] **Step 1: Add `content_type_filter` to `SearchParams`**

In `frontend/src/lib/types.ts`, add to the `SearchParams` interface after `lyti_filter`:

```typescript
content_type_filter?: string;
```

- [ ] **Step 2: Wire `content_type_filter` in `api.ts`**

In `frontend/src/lib/api.ts`, add after the `lyti_filter` block (after line 53):

```typescript
if (params.content_type_filter) {
    searchParams.append('content_type_filter', params.content_type_filter);
}
```

- [ ] **Step 3: Pass `content_type_filter` and `limit` from the search page loader**

In `frontend/src/routes/search/+page.ts`, update the params object:

```typescript
const params: SearchParams = {
    query,
    limit: 20,
    league_filter: league,
    year_filter: year,
    team_filter: team,
    content_type_filter: 'text'
};
```

- [ ] **Step 4: Verify the frontend builds**

Run: `cd frontend && npm run build`
Expected: Build succeeds with no errors.

- [ ] **Step 5: Commit**

```bash
git add frontend/src/lib/types.ts frontend/src/lib/api.ts frontend/src/routes/search/+page.ts
git commit -m "feat: filter web search to text-only content and increase limit to 20"
```

---

### Task 3: Render clickable breadcrumbs on chunk results

**Files:**
- Modify: `frontend/src/lib/components/ChunkResult.svelte`
- Modify: `frontend/src/lib/components/PaperGroup.svelte`

- [ ] **Step 1: Update ChunkResult to accept and render breadcrumbs**

Replace the full content of `frontend/src/lib/components/ChunkResult.svelte`:

```svelte
<script lang="ts">
	import type { BreadcrumbEntry } from '$lib/types';
	import { slugifyHeading } from '$lib/markdown';

	interface Props {
		text: string;
		query: string;
		score: number;
		breadcrumbs?: BreadcrumbEntry[];
		title?: string;
		paperId: string;
	}

	let { text, query, score, breadcrumbs, title, paperId }: Props = $props();

	function highlightText(text: string, query: string): string {
		if (!query.trim()) return text;

		const words = query
			.trim()
			.split(/\s+/)
			.filter((w) => w.length > 0);

		let result = text;
		for (const word of words) {
			const regex = new RegExp(`(${escapeRegex(word)})`, 'gi');
			result = result.replace(regex, '<mark class="bg-yellow-200">$1</mark>');
		}

		return result;
	}

	function escapeRegex(str: string): string {
		return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
	}

	const highlighted = $derived(highlightText(text, query));

	// Build full breadcrumb trail: ancestors + chunk's own title
	const crumbs = $derived.by(() => {
		const trail: { title: string; href: string }[] = [];
		for (const b of breadcrumbs ?? []) {
			trail.push({
				title: b.title,
				href: `/paper/${paperId}#${slugifyHeading(b.title)}`
			});
		}
		if (title) {
			trail.push({
				title,
				href: `/paper/${paperId}#${slugifyHeading(title)}`
			});
		}
		return trail;
	});
</script>

<div class="text-xs sm:text-sm text-gray-700 leading-relaxed">
	{#if crumbs.length > 0}
		<div class="text-xs text-gray-400 mb-1">
			{#each crumbs as crumb, i}
				{#if i > 0}
					<span class="mx-1">&gt;</span>
				{/if}
				<a href={crumb.href} class="hover:text-blue-600 hover:underline">{crumb.title}</a>
			{/each}
		</div>
	{/if}
	<div class="flex items-start justify-between gap-2 mb-1">
		<div class="flex-1 min-w-0 break-words">
			{@html highlighted}
		</div>
		<span class="text-xs text-gray-500 font-mono flex-shrink-0">
			{score.toFixed(3)}
		</span>
	</div>
</div>
```

- [ ] **Step 2: Update PaperGroup to pass breadcrumb data to ChunkResult**

In `frontend/src/lib/components/PaperGroup.svelte`, update the two `ChunkResult` usages.

The first `{#each topChunks as chunk}` block (around line 56-62), change to:

```svelte
{#each topChunks as chunk}
    <ChunkResult
        text={chunk.text}
        {query}
        score={chunk.score}
        breadcrumbs={chunk.breadcrumbs}
        title={chunk.title}
        {paperId}
    />
{/each}
```

The second `{#each remainingChunks as chunk}` block (around line 66-72), change to:

```svelte
{#each remainingChunks as chunk}
    <ChunkResult
        text={chunk.text}
        {query}
        score={chunk.score}
        breadcrumbs={chunk.breadcrumbs}
        title={chunk.title}
        {paperId}
    />
{/each}
```

- [ ] **Step 3: Verify the frontend builds**

Run: `cd frontend && npm run build`
Expected: Build succeeds with no errors.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/components/ChunkResult.svelte frontend/src/lib/components/PaperGroup.svelte
git commit -m "feat: render clickable breadcrumbs on search result chunks"
```
