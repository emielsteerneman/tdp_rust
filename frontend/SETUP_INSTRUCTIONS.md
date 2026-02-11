# Frontend Setup Instructions

The SvelteKit 5 project has been scaffolded with all necessary configuration files. Due to permission restrictions, npm commands could not be executed automatically.

## Manual Steps Required

To complete the setup, run the following commands:

```bash
cd /home/emiel/projects/tdp_rust/frontend
npm install
npm run dev
```

## What Was Created

### Project Structure
```
frontend/
├── src/
│   ├── lib/
│   │   ├── api.ts          # Typed API wrappers for all 6 endpoints
│   │   └── types.ts        # TypeScript interfaces mirroring Rust DTOs
│   ├── routes/
│   │   ├── +layout.svelte  # Root layout with CSS import
│   │   └── +page.svelte    # Home page
│   ├── app.css             # Tailwind CSS v4 import
│   └── app.html            # HTML template
├── static/                 # Static assets directory
├── package.json            # Dependencies and scripts
├── svelte.config.js        # SvelteKit configuration
├── vite.config.ts          # Vite config with /api proxy to localhost:8080
├── tsconfig.json           # TypeScript configuration
├── .gitignore              # Git ignore patterns
└── README.md               # Project documentation
```

### Dependencies Configured

**DevDependencies:**
- @sveltejs/adapter-auto: ^3.0.0
- @sveltejs/kit: ^2.0.0
- @sveltejs/vite-plugin-svelte: ^4.0.0
- @tailwindcss/typography: ^0.5.0
- svelte: ^5.0.0
- svelte-check: ^4.0.0
- tailwindcss: ^4.0.0
- typescript: ^5.0.0
- vite: ^6.0.0

**Dependencies:**
- marked: ^12.0.0

### TypeScript Types (src/lib/types.ts)

Mirrored all Rust data structures:
- League
- TeamName
- TDPName
- SearchResultChunk
- ScoredChunk
- SearchResult
- SearchSuggestions
- Filter
- EmbedType
- SearchParams
- ApiResponse
- Paper

### API Wrappers (src/lib/api.ts)

All 6 endpoints are implemented:

1. **search(params: SearchParams): Promise<SearchResult>**
   - GET /api/search
   - Supports query, limit, league_filter, year_filter, team_filter, lyti_filter, search_type

2. **listPapers(): Promise<TDPName[]>**
   - GET /api/papers

3. **getPaper(league, year, team, index): Promise<Paper>**
   - GET /api/papers/:id
   - Returns markdown content

4. **getPaperByParams(league, year, team): Promise<Paper>**
   - GET /api/paper (alternative endpoint with query params)

5. **listTeams(hint?: string): Promise<TeamName[]>**
   - GET /api/teams
   - Optional hint parameter for filtering

6. **listLeagues(): Promise<League[]>**
   - GET /api/leagues

7. **listYears(): Promise<number[]>**
   - GET /api/years

### Vite Proxy Configuration

The vite.config.ts is configured to proxy all `/api/*` requests to `http://localhost:8080`, which is where the Axum backend runs.

### Tailwind CSS v4

The project uses Tailwind CSS v4 with the new `@import "tailwindcss"` syntax (not the v3 `@tailwind` directives). The `@tailwindcss/typography` plugin is included in dependencies.

## Verification

After running `npm install`, verify the setup with:

```bash
npm run dev
```

This should start the dev server on http://localhost:5173 without errors. The proxy won't be testable until the backend is running, but the frontend should compile and serve correctly.

## Notes

- The backend routes are not yet implemented in the web crate, but the API wrappers are ready for when they are.
- The TypeScript types exactly match the Rust DTOs from data_structures and web/src/dto.rs.
- SvelteKit 5 conventions are used throughout (Svelte 5 with runes support).
