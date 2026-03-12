# Unified Filter Layout Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Unify the browse and search pages under a shared layout with a persistent FilterSidebar and SearchBar, so filters and search queries are preserved across navigation.

**Architecture:** SvelteKit route group `(browse)` provides a shared layout rendering FilterSidebar + content slot. URL search params are the single source of truth for filter and query state. The search page reads multi-value filter params and translates machine names to pretty names for the backend API.

**Tech Stack:** SvelteKit 2, Svelte 5 (runes), TypeScript

**Spec:** `docs/superpowers/specs/2026-03-12-unified-filter-layout-design.md`

---

## Chunk 1: Route restructure and shared layout

### Task 1: Create the (browse) route group and move files

This task restructures routes without changing any behavior. After this, the app works exactly as before but files live under `(browse)/`.

**Files:**
- Create: `frontend/src/routes/(browse)/+layout.svelte`
- Create: `frontend/src/routes/(browse)/+page.ts`
- Move: `frontend/src/routes/+page.svelte` → `frontend/src/routes/(browse)/+page.svelte`
- Move: `frontend/src/routes/search/+page.svelte` → `frontend/src/routes/(browse)/search/+page.svelte`
- Move: `frontend/src/routes/search/+page.ts` → `frontend/src/routes/(browse)/search/+page.ts`

- [ ] **Step 1: Create the (browse) directory structure**

```bash
mkdir -p frontend/src/routes/'(browse)'/search
```

- [ ] **Step 2: Move the home page into the route group**

```bash
mv frontend/src/routes/+page.svelte frontend/src/routes/'(browse)'/+page.svelte
```

- [ ] **Step 3: Create a minimal +page.ts for the browse page**

The old `routes/+page.ts` just set SSR options. Create a new one in the group:

```typescript
// frontend/src/routes/(browse)/+page.ts
export const prerender = false;
export const ssr = true;
```

Delete the old one:
```bash
rm frontend/src/routes/+page.ts
```

- [ ] **Step 4: Move the search route into the group**

```bash
mv frontend/src/routes/search/+page.svelte frontend/src/routes/'(browse)'/search/+page.svelte
mv frontend/src/routes/search/+page.ts frontend/src/routes/'(browse)'/search/+page.ts
rmdir frontend/src/routes/search
```

- [ ] **Step 5: Create the (browse) group layout**

This layout extracts the FilterSidebar from the home page and wraps the content slot. The home page currently renders `<FilterSidebar>` + `<main>` in a flex row — we move that structure here.

Note: This layout has no `+layout.ts` of its own. `LayoutData` includes data from the root `+layout.ts` (papers, teams, leagues, years) via SvelteKit's automatic data cascading. Uses Svelte 5 `children` snippet pattern (the root layout still uses the Svelte 4 `<slot/>` — both work, consistency update is optional).

```svelte
<!-- frontend/src/routes/(browse)/+layout.svelte -->
<script lang="ts">
	import FilterSidebar from '$lib/components/FilterSidebar.svelte';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: any } = $props();
</script>

<div class="flex flex-col lg:flex-row">
	<FilterSidebar
		leagues={data.leagues}
		years={data.years}
		teams={data.teams}
	/>

	<main class="flex-1 min-w-0">
		{@render children()}
	</main>
</div>
```

- [ ] **Step 6: Remove FilterSidebar rendering from the browse page**

Edit `frontend/src/routes/(browse)/+page.svelte`:
- Remove the `FilterSidebar` import
- Remove the `<FilterSidebar>` component and the wrapping `<div class="flex flex-col lg:flex-row">` and `<main>` tags
- Keep only the inner content (intro blurb + paper grid)
- Update the Props interface — the page still receives `data` from the root layout (papers, teams, leagues, years) but no longer needs to pass leagues/years/teams to FilterSidebar

The page should look like:

```svelte
<script lang="ts">
	import { page } from '$app/stores';
	import PaperCard from '$lib/components/PaperCard.svelte';
	import type { TDPName, League, TeamName } from '$lib/types';

	interface Props {
		data: {
			papers: TDPName[];
			teams: TeamName[];
			leagues: League[];
			years: number[];
		};
	}

	let { data }: Props = $props();

	let selectedLeagues = $derived($page.url.searchParams.getAll('league'));
	let selectedYears = $derived($page.url.searchParams.getAll('year').map(y => parseInt(y)));
	let selectedTeams = $derived($page.url.searchParams.getAll('team'));

	let filteredPapers = $derived(
		data.papers
			.filter((paper) => {
				if (selectedLeagues.length > 0 && !selectedLeagues.includes(paper.league.name)) {
					return false;
				}
				if (selectedYears.length > 0 && !selectedYears.includes(paper.year)) {
					return false;
				}
				if (selectedTeams.length > 0 && !selectedTeams.includes(paper.team_name.name)) {
					return false;
				}
				return true;
			})
			.sort((a, b) => b.year - a.year)
	);

	let groupedPapers = $derived.by(() => {
		const yearMap = new Map<number, Map<string, TDPName[]>>();
		for (const paper of filteredPapers) {
			let leagueMap = yearMap.get(paper.year);
			if (!leagueMap) {
				leagueMap = new Map();
				yearMap.set(paper.year, leagueMap);
			}
			let papers = leagueMap.get(paper.league.name);
			if (!papers) {
				papers = [];
				leagueMap.set(paper.league.name, papers);
			}
			papers.push(paper);
		}
		return [...yearMap.entries()]
			.sort(([a], [b]) => b - a)
			.map(([year, leagueMap]) => ({
				year,
				leagues: [...leagueMap.entries()]
					.sort(([a], [b]) => a.localeCompare(b))
					.map(([, papers]) => ({
						league: papers[0].league,
						papers
					}))
			}));
	});

	function getLeagueBadgeColor(leagueName: string): string {
		if (leagueName.includes('smallsize')) return 'bg-blue-100 text-blue-800';
		if (leagueName.includes('middlesize')) return 'bg-green-100 text-green-800';
		if (leagueName.includes('humanoid')) return 'bg-purple-100 text-purple-800';
		if (leagueName.includes('standard_platform')) return 'bg-orange-100 text-orange-800';
		if (leagueName.includes('rescue')) return 'bg-red-100 text-red-800';
		if (leagueName.includes('athome')) return 'bg-yellow-100 text-yellow-800';
		if (leagueName.includes('industrial')) return 'bg-gray-100 text-gray-800';
		return 'bg-gray-100 text-gray-800';
	}

	let hasActiveFilters = $derived(
		selectedLeagues.length > 0 || selectedYears.length > 0 || selectedTeams.length > 0
	);

	let isLoading = $state(false);
</script>

<!-- Introduction (desktop only) -->
<div class="hidden md:block bg-gradient-to-b from-blue-50 to-white py-8 px-4">
	<p class="max-w-3xl mx-auto text-gray-600 text-base leading-relaxed">
		Opponents on the field, and colleagues next to it. Not just us, but all before us through the knowledge in their TDPs. Over 2000 and counting. Reading 2000 papers is of course impossible. Therefore, to keep our inspiration and innovation going, I made this information more accessible through this TDP Search Engine.
	</p>
</div>

<!-- Papers -->
<div class="max-w-7xl mx-auto px-4 py-6 sm:py-8">
	<div class="mb-6">
		<h2 class="text-xl sm:text-2xl font-semibold text-gray-900">
			{#if hasActiveFilters}
				Filtered Papers
			{:else}
				All Papers
			{/if}
		</h2>
		<p class="text-sm text-gray-600 mt-1">
			Showing {filteredPapers.length} of {data.papers.length} papers
		</p>
	</div>

	{#if isLoading}
		<div class="flex justify-center items-center py-16">
			<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
		</div>
	{:else if filteredPapers.length === 0}
		<div class="text-center py-12 bg-gray-50 rounded-lg">
			<svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
			</svg>
			<p class="text-gray-500 text-lg mb-2">
				No papers match the selected filters.
			</p>
			{#if hasActiveFilters}
				<p class="text-sm text-gray-400">
					Try adjusting your filters to see more results.
				</p>
			{/if}
		</div>
	{:else}
		<div class="space-y-8">
			{#each groupedPapers as yearGroup}
				<section>
					<h3 class="text-2xl sm:text-3xl font-bold text-gray-900 border-b border-gray-200 pb-2 mb-4">
						{yearGroup.year}
					</h3>
					<div class="space-y-4">
						{#each yearGroup.leagues as leagueGroup}
							<div>
								<div class="flex items-center gap-2 mb-2">
									<span class="px-3 py-1 text-base font-semibold rounded-full {getLeagueBadgeColor(leagueGroup.league.name)}">
										{leagueGroup.league.name_pretty}
									</span>
								</div>
								<div class="flex flex-wrap gap-2">
									{#each leagueGroup.papers as paper}
										<PaperCard {paper} />
									{/each}
								</div>
							</div>
						{/each}
					</div>
				</section>
			{/each}
		</div>
	{/if}
</div>
```

- [ ] **Step 7: Build and verify**

```bash
cd frontend && npm run build
```

Expected: Build succeeds. URLs `/` and `/search` work as before. FilterSidebar appears on both pages.

- [ ] **Step 8: Commit**

```bash
git add -A frontend/src/routes/
git commit -m "refactor: move browse and search routes into (browse) layout group

Extract FilterSidebar into shared (browse) group layout so it persists
across browse and search views."
```

---

## Chunk 2: Fix FilterSidebar, SearchBar, and Navbar for cross-page state

### Task 2: Fix FilterSidebar.clearAllFilters() to preserve query param

**Files:**
- Modify: `frontend/src/lib/components/FilterSidebar.svelte`

- [ ] **Step 1: Update clearAllFilters()**

In `FilterSidebar.svelte`, change `clearAllFilters()` from:

```typescript
function clearAllFilters() {
    goto('/', { replaceState: true });
}
```

To:

```typescript
function clearAllFilters() {
    const params = new URLSearchParams($page.url.searchParams);
    params.delete('league');
    params.delete('year');
    params.delete('team');
    const qs = params.toString();
    goto(`${$page.url.pathname}${qs ? '?' + qs : ''}`, { replaceState: true, keepFocus: true });
}
```

This preserves the current path (works on both `/` and `/search`) and preserves the `q=` param if present.

- [ ] **Step 2: Build and verify**

```bash
cd frontend && npm run build
```

Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/lib/components/FilterSidebar.svelte
git commit -m "fix: FilterSidebar.clearAllFilters preserves path and query param"
```

### Task 3: Update SearchBar to preserve filter params

**Files:**
- Modify: `frontend/src/lib/components/SearchBar.svelte`

- [ ] **Step 1: Import $page and preserve filters on search**

Replace the full `SearchBar.svelte` content with:

```svelte
<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';

	interface Props {
		initialValue?: string;
		compact?: boolean;
	}

	let { initialValue = '', compact = false }: Props = $props();

	let query = $state('');

	$effect(() => {
		query = initialValue;
	});

	function handleSearch() {
		if (query.trim()) {
			const params = new URLSearchParams();
			params.set('q', query.trim());
			// Preserve filter params from current URL
			for (const key of ['league', 'year', 'team']) {
				for (const value of $page.url.searchParams.getAll(key)) {
					params.append(key, value);
				}
			}
			goto(`/search?${params.toString()}`);
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			handleSearch();
		}
	}
</script>

<div class="relative w-full {compact ? 'max-w-md' : 'max-w-2xl'}">
	<div class="relative">
		<input
			type="text"
			bind:value={query}
			onkeydown={handleKeydown}
			placeholder="Search TDP papers..."
			class="w-full px-4 py-2 pl-10 pr-4 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
		/>
		<svg
			class="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-400"
			fill="none"
			stroke="currentColor"
			viewBox="0 0 24 24"
		>
			<path
				stroke-linecap="round"
				stroke-linejoin="round"
				stroke-width="2"
				d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
			/>
		</svg>
	</div>
</div>
```

- [ ] **Step 2: Build and verify**

```bash
cd frontend && npm run build
```

Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/lib/components/SearchBar.svelte
git commit -m "feat: SearchBar preserves filter params when searching"
```

### Task 4: Update Navbar to pass query to SearchBar

**Files:**
- Modify: `frontend/src/lib/components/Navbar.svelte`

- [ ] **Step 1: Import $page and pass q param to SearchBar**

Replace the full `Navbar.svelte` content with:

```svelte
<script lang="ts">
	import { page } from '$app/stores';
	import SearchBar from './SearchBar.svelte';

	let currentQuery = $derived($page.url.searchParams.get('q') || '');
</script>

<nav class="sticky top-0 z-50 bg-white border-b border-gray-200 shadow-sm">
	<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
		<div class="flex items-center justify-between h-16">
			<!-- Logo/Title -->
			<div class="flex-shrink-0">
				<a href="/" class="flex items-center space-x-2">
					<span class="text-xl font-bold text-gray-900">TDP Browser</span>
				</a>
			</div>

			<!-- Search Bar (desktop) -->
			<div class="hidden md:flex flex-1 justify-center px-8">
				<SearchBar compact={true} initialValue={currentQuery} />
			</div>
		</div>

		<!-- Search Bar (mobile - always visible) -->
		<div class="md:hidden pb-3">
			<SearchBar compact={false} initialValue={currentQuery} />
		</div>
	</div>
</nav>
```

- [ ] **Step 2: Build and verify**

```bash
cd frontend && npm run build
```

Expected: Build succeeds. Searching preserves filters. Query persists in SearchBar.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/lib/components/Navbar.svelte
git commit -m "feat: Navbar passes current query to SearchBar for persistence"
```

---

## Chunk 3: Rewrite search page for multi-select filters and open papers in new tabs

### Task 5: Update search/+page.ts to read multi-value filter params

**Files:**
- Modify: `frontend/src/routes/(browse)/search/+page.ts`

- [ ] **Step 1: Rewrite the page loader**

Replace the full file with:

```typescript
import type { PageLoad } from './$types';
import { search } from '$lib/api';
import type { SearchParams, League, TeamName } from '$lib/types';

export const load: PageLoad = async ({ url, fetch, parent }) => {
	const query = url.searchParams.get('q') || '';

	if (!query) {
		return {
			searchResult: null,
			query: '',
			loading: false
		};
	}

	// Get layout data for name translation (machine name -> pretty name)
	const layoutData = await parent();

	// Read multi-value filter params (machine names from URL)
	const leagueMachineNames = url.searchParams.getAll('league');
	const yearStrings = url.searchParams.getAll('year');
	const teamMachineNames = url.searchParams.getAll('team');

	// Translate machine names to pretty names for the backend API
	const leaguePrettyNames = leagueMachineNames
		.map((name: string) => (layoutData.leagues as League[]).find((l: League) => l.name === name)?.name_pretty)
		.filter((n): n is string => n !== undefined);

	const teamPrettyNames = teamMachineNames
		.map((name: string) => (layoutData.teams as TeamName[]).find((t: TeamName) => t.name === name)?.name_pretty)
		.filter((n): n is string => n !== undefined);

	const params: SearchParams = {
		query,
		limit: 20,
		league_filter: leaguePrettyNames.length > 0 ? leaguePrettyNames.join(', ') : undefined,
		year_filter: yearStrings.length > 0 ? yearStrings.join(', ') : undefined,
		team_filter: teamPrettyNames.length > 0 ? teamPrettyNames.join(', ') : undefined,
		content_type_filter: 'text'
	};

	try {
		const searchResult = await search(params, fetch);
		return {
			searchResult,
			query,
			loading: false
		};
	} catch (error) {
		console.error('Search error:', error);
		return {
			searchResult: null,
			query,
			error: error instanceof Error ? error.message : 'Search failed',
			loading: false
		};
	}
};
```

- [ ] **Step 2: Build and verify**

```bash
cd frontend && npm run build
```

Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/routes/'(browse)'/search/+page.ts
git commit -m "feat: search page reads multi-value filter params with name translation"
```

### Task 6: Rewrite search/+page.svelte (strip inline filters)

**Files:**
- Modify: `frontend/src/routes/(browse)/search/+page.svelte`

- [ ] **Step 1: Rewrite the search results page**

The page no longer has its own filter sidebar. It just renders the heading, result count, and paper groups. Replace the full file with:

```svelte
<script lang="ts">
	import type { PageData } from './$types';
	import type { SearchResultChunk } from '$lib/types';
	import PaperGroup from '$lib/components/PaperGroup.svelte';
	import { page } from '$app/stores';
	import { navigating } from '$app/stores';

	let { data }: { data: PageData } = $props();
	let isNavigating = $derived($navigating !== null);

	interface PaperGroupData {
		paperId: string;
		chunks: SearchResultChunk[];
		avgScore: number;
	}

	const paperGroups = $derived.by(() => {
		if (!data.searchResult?.chunks) return [];

		const groups = new Map<string, SearchResultChunk[]>();

		for (const chunk of data.searchResult.chunks) {
			const paperId = chunk.league_year_team_idx;
			if (!groups.has(paperId)) {
				groups.set(paperId, []);
			}
			groups.get(paperId)!.push(chunk);
		}

		const groupArray: PaperGroupData[] = [];
		for (const [paperId, chunks] of groups) {
			const avgScore = chunks.reduce((sum, c) => sum + c.score, 0) / chunks.length;
			groupArray.push({ paperId, chunks, avgScore });
		}

		groupArray.sort((a, b) => b.avgScore - a.avgScore);

		return groupArray;
	});

	let hasFilters = $derived(
		$page.url.searchParams.has('league') ||
		$page.url.searchParams.has('year') ||
		$page.url.searchParams.has('team')
	);
</script>

<div class="max-w-7xl mx-auto px-4 py-6 sm:py-8">
	{#if data.error}
		<div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-6">
			<p class="text-red-800">Error: {data.error}</p>
		</div>
	{/if}

	{#if !data.query}
		<div class="text-center py-12">
			<h1 class="text-xl sm:text-2xl font-semibold text-gray-900 mb-4">Search TDP Papers</h1>
			<p class="text-gray-600">Enter a search query to get started.</p>
		</div>
	{:else}
		<div class="mb-4 sm:mb-6">
			<h1 class="text-xl sm:text-2xl font-semibold text-gray-900 mb-2">
				Search Results for "{data.query}"
			</h1>
			{#if data.searchResult}
				<p class="text-sm sm:text-base text-gray-600">
					Found {data.searchResult.chunks.length} {data.searchResult.chunks.length === 1
						? 'result'
						: 'results'} in {paperGroups.length} {paperGroups.length === 1
						? 'paper'
						: 'papers'}
				</p>
			{/if}
		</div>

		{#if isNavigating}
			<div class="flex justify-center items-center py-16">
				<div class="text-center">
					<div class="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
					<p class="text-gray-600">Loading results...</p>
				</div>
			</div>
		{:else if data.searchResult && data.searchResult.chunks.length > 0}
			<div class="space-y-4">
				{#each paperGroups as group (group.paperId)}
					<PaperGroup
						paperId={group.paperId}
						chunks={group.chunks}
						query={data.query}
					/>
				{/each}
			</div>
		{:else if data.searchResult}
			<div class="bg-white border border-gray-200 rounded-lg p-6 sm:p-8 text-center">
				<svg class="mx-auto h-12 w-12 text-gray-400 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
				</svg>
				<p class="text-gray-600 mb-2">No results found for "{data.query}"</p>
				{#if hasFilters}
					<p class="text-sm text-gray-500 mt-2">
						Try removing some filters to see more results.
					</p>
				{/if}
			</div>
		{/if}
	{/if}
</div>
```

Note: `paperGroups` changed from a function (`$derived(() => ...)` called as `paperGroups()`) to a direct derived value (`$derived.by(() => ...)` accessed as `paperGroups`). This fixes a subtle Svelte 5 pattern issue in the original code.

- [ ] **Step 2: Build and verify**

```bash
cd frontend && npm run build
```

Expected: Build succeeds.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/routes/'(browse)'/search/+page.svelte
git commit -m "refactor: strip inline filters from search page, use shared sidebar"
```

### Task 7: Open paper links in new tabs

**Files:**
- Modify: `frontend/src/lib/components/PaperCard.svelte`

- [ ] **Step 1: Add target="_blank" to the paper link**

In `PaperCard.svelte`, change the `<a>` tag from:

```svelte
<a
	href="/paper/{paperId}"
	class="inline-block px-3 py-1 text-sm rounded-lg border border-gray-200 bg-white text-gray-800 hover:border-blue-300 hover:bg-blue-50 transition-colors"
>
```

To:

```svelte
<a
	href="/paper/{paperId}"
	target="_blank"
	class="inline-block px-3 py-1 text-sm rounded-lg border border-gray-200 bg-white text-gray-800 hover:border-blue-300 hover:bg-blue-50 transition-colors"
>
```

- [ ] **Step 2: Add target="_blank" to PaperGroup paper link**

In `frontend/src/lib/components/PaperGroup.svelte`, line 34-36, change:

```svelte
				<a
					href="/paper/{paperId}"
					class="text-base sm:text-lg font-semibold text-blue-600 hover:text-blue-800 hover:underline break-words"
				>
```

To:

```svelte
				<a
					href="/paper/{paperId}"
					target="_blank"
					class="text-base sm:text-lg font-semibold text-blue-600 hover:text-blue-800 hover:underline break-words"
				>
```

- [ ] **Step 3: Build and verify**

```bash
cd frontend && npm run build
```

Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/lib/components/PaperCard.svelte frontend/src/lib/components/PaperGroup.svelte
git commit -m "feat: open paper links in new tabs"
```

### Task 8: Final build verification

- [ ] **Step 1: Clean build**

```bash
cd frontend && rm -rf .svelte-kit build && npm run build
```

Expected: Build succeeds with no errors or warnings.

- [ ] **Step 2: Verify route structure**

Check that the built output includes all expected routes:
```bash
ls -la frontend/build/
```

- [ ] **Step 3: Commit if there are any remaining changes**

```bash
git status
# If clean, nothing to do. If there are changes, commit them.
```
