# Unified Filter Layout Design

## Problem

The browse page and search page have independent, disconnected filter systems. Filters set on the home page are lost when searching. The search page has its own simpler single-select filters. Re-searching from the navbar drops active filters. Users can't maintain filter context across browse/search.

## Goals

- Persistent FilterSidebar + SearchBar across browse and search views
- Filters are global state (URL params) shared between browse and search
- Multi-select filters on search (matching browse behavior)
- Search preserves active filters; filters preserve active search query
- Paper pages open in new tabs and don't render the sidebar

## Route Structure

```
routes/
  +layout.svelte          # Navbar only (all pages)
  +layout.ts              # Loads papers/teams/leagues/years (unchanged)
  (browse)/
    +layout.svelte        # NEW: FilterSidebar + main content <slot/>
    +page.svelte          # MOVED: paper grid (browse view)
    search/
      +page.svelte        # REWRITTEN: search results (no own filters)
      +page.ts            # UPDATED: reads multi-value filter params
  paper/[id]/
    +page.svelte          # Unchanged
    +page.ts              # Unchanged
```

The `(browse)` route group doesn't affect URLs. `/` and `/search` remain the same. `/paper/[id]` stays outside the group.

No `(browse)/+page.ts` is needed. The current `routes/+page.ts` only sets `prerender = false` and `ssr = true` — these can be inlined in the moved `+page.svelte` or a minimal `+page.ts`. The browse page gets its data (papers, teams, leagues, years) from the root `+layout.ts` via SvelteKit's layout data inheritance.

No `(browse)/+layout.ts` is needed either — the group layout inherits data from the root layout.

## Filter State: URL as Single Source of Truth

All filter state lives in URL search params. **Machine names** are used consistently in URL params (e.g., `league=soccer_smallsize`, `team=RoboTeam_Twente`), matching the current FilterSidebar behavior.

Example: `?league=soccer_smallsize&league=soccer_humanoid&year=2024&team=RoboTeam_Twente&q=trajectory+planning`

Both FilterSidebar and SearchBar read from and write to these params.

### Name Translation: Machine Names vs Pretty Names

The FilterSidebar writes **machine names** (`league.name`, `team.name`) to URL params. The browse page filters locally against `paper.league.name` (machine name) — no translation needed.

The backend search API expects **pretty names** (`Soccer SmallSize`, `RoboTeam Twente`). Therefore, `search/+page.ts` must translate machine names from URL params to pretty names before calling the API. This is done by looking up the league/team objects from the layout data (available via `parent()`) and mapping `name` to `name_pretty`.

### FilterSidebar Changes

- **`clearAllFilters()`**: currently hardcodes `goto('/')`. Must change to stay on the current path and only strip league/year/team params while preserving `q=` and any other params. This is a behavioral change, not just a tweak.
- **`toggleFilter()` and `toggleGroupLeagues()`**: currently use relative `goto('?...')` which correctly stays on the current path. The param-building logic already uses `$page.url.searchParams` so existing params (including `q=`) are preserved. No changes needed to these functions.

### SearchBar Changes

- **On search**: navigate to `/search?q=...` while preserving existing league/year/team params from the current URL. SearchBar must import `$page` store directly (from `$app/stores`) to read current URL params — it currently doesn't have access.
- **Query persistence**: SearchBar reads `initialValue` from the current URL's `q` param. Navbar must import `$page` store and pass `$page.url.searchParams.get('q')` as `initialValue` to SearchBar.

### (browse)/+layout.svelte

Renders the FilterSidebar + `<slot/>` in a flex row (same layout the home page currently uses). FilterSidebar receives leagues/years/teams from the parent layout data.

## Backend Integration

No backend changes needed. The search API (`SearchArgs`) already supports comma-separated multi-value filters:
- `league_filter=Soccer SmallSize, Soccer Humanoid`
- `year_filter=2023, 2024`
- `team_filter=RoboTeam Twente, TIGERs Mannheim`

### Frontend API Changes

`search/+page.ts`:
- Read filter params with `url.searchParams.getAll('league')` etc. to get arrays of machine names
- Look up corresponding pretty names from parent layout data
- Join with `, ` to create comma-separated strings for the API
- Pass to `search()` as `league_filter`, `year_filter`, `team_filter`

`api.ts search()`: no changes needed — it already passes string values through.

## Content Area Behavior

- **No `q` param (browse mode):** Show intro blurb + paper grid filtered by active league/year/team params
- **With `q` param (search mode):** Show search results filtered by active league/year/team params
- **`/?q=foo`**: This URL pattern is ignored — `q` param only has meaning on `/search`. The browse page does not read `q`.
- **Paper links:** open in new tab (`target="_blank"`) — requires adding `target="_blank"` to PaperCard links

## Search Results Page Simplification

The current `search/+page.svelte` has its own inline filter sidebar (single-select dropdowns). This is removed entirely — the shared FilterSidebar in `(browse)/+layout.svelte` handles all filtering.

The search page becomes just: heading + result count + paper groups.

## Files Changed

- `FilterSidebar.svelte`: fix `clearAllFilters()` to preserve `q=` param and stay on current path
- `SearchBar.svelte`: import `$page`, preserve filter params on search navigation
- `Navbar.svelte`: import `$page`, pass current `q` param to SearchBar as `initialValue`
- `PaperCard.svelte`: add `target="_blank"` to paper links
- `(browse)/+layout.svelte`: new file, renders FilterSidebar + slot
- `(browse)/+page.svelte`: moved from `routes/+page.svelte`
- `(browse)/search/+page.svelte`: rewritten (stripped of inline filters)
- `(browse)/search/+page.ts`: updated to read multi-value params and translate names

## Files Unchanged

- `paper/[id]` route: untouched, outside the layout group
- Root `+layout.svelte`: still renders Navbar + `<slot/>`
- Root `+layout.ts`: still loads papers/teams/leagues/years
- `PaperGroup.svelte`, `ChunkResult.svelte`: unchanged
- `api.ts`: unchanged
- All backend code: unchanged
