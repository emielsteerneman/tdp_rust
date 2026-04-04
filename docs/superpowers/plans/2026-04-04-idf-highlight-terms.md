# IDF-Based Highlight Terms Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Return IDF-filtered highlight terms from backend search results so the frontend only highlights meaningful query terms.

**Architecture:** Add `extract_highlight_terms()` in `data_access`, a new `highlight_terms` field on `SearchResult`, a configurable IDF threshold in `DataProcessingConfig`, and update three frontend components to consume the new field instead of splitting the raw query string.

**Tech Stack:** Rust (data_access, data_structures, data_processing, configuration crates), TypeScript/Svelte 5 (frontend)

**Spec:** `docs/superpowers/specs/2026-04-04-idf-highlight-terms-design.md`

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `data_access/src/embed/mod.rs` | Modify | Add `extract_highlight_terms()` function |
| `data_structures/src/intermediate/search.rs` | Modify | Add `highlight_terms` field to `SearchResult` |
| `data_processing/src/config.rs` | Modify | Add optional `highlight_idf_threshold` config field |
| `data_processing/src/search.rs` | Modify | Add threshold to `Searcher`, call `extract_highlight_terms()` |
| `web/src/main.rs` | Modify | Pass config threshold to `Searcher::new()` |
| `mcp/src/main.rs` | Modify | Pass config threshold to `Searcher::new()` |
| `tools/src/bin/smoke_test.rs` | Modify | Pass config threshold to `Searcher::new()` |
| `tools/src/bin/search_by_sentence.rs` | Modify | Pass config threshold to `Searcher::new()` |
| `config.toml.example` | Modify | Add commented `highlight_idf_threshold` example |
| `frontend/src/lib/types.ts` | Modify | Add `highlight_terms` to `SearchResult` interface |
| `frontend/src/routes/(browse)/search/+page.svelte` | Modify | Pass `highlightTerms` instead of `query` to `PaperGroup` |
| `frontend/src/lib/components/PaperGroup.svelte` | Modify | Accept and forward `highlightTerms` prop |
| `frontend/src/lib/components/ChunkResult.svelte` | Modify | Accept `highlightTerms`, sort by length descending, use directly in regex |

---

### Task 1: Add `extract_highlight_terms()` function

**Files:**
- Modify: `data_access/src/embed/mod.rs:41-54`

- [ ] **Step 1: Write the test**

Add a test module at the bottom of `data_access/src/embed/mod.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use data_structures::IDF;

    #[test]
    fn test_extract_highlight_terms_filters_by_base_idf() {
        // Simulate IDF map with:
        // - "robot" as a common unigram (base_idf 1.2, weighted 1.2) — below threshold
        // - "solenoid" as a rare unigram (base_idf 3.5, weighted 3.5) — above threshold
        // - "solenoid winder" as a rare bigram (base_idf 3.8, weighted 7.6) — above threshold
        let idf_map = IDF::from([
            ("robot".to_string(), (0, 1.2)),
            ("solenoid".to_string(), (1, 3.5)),
            ("solenoid winder".to_string(), (2, 7.6)),
        ]);

        let terms = extract_highlight_terms("robot solenoid winder", &idf_map, 1.5);

        assert!(terms.contains(&"solenoid".to_string()));
        assert!(terms.contains(&"solenoid winder".to_string()));
        assert!(!terms.contains(&"robot".to_string()));
    }

    #[test]
    fn test_extract_highlight_terms_sorted_by_weighted_idf_descending() {
        let idf_map = IDF::from([
            ("winder".to_string(), (0, 2.0)),
            ("solenoid".to_string(), (1, 3.5)),
            ("solenoid winder".to_string(), (2, 7.6)),
        ]);

        let terms = extract_highlight_terms("solenoid winder", &idf_map, 1.5);

        // "solenoid winder" (7.6) should come before "solenoid" (3.5), then "winder" (2.0)
        assert_eq!(terms[0], "solenoid winder");
        assert_eq!(terms[1], "solenoid");
        assert_eq!(terms[2], "winder");
    }

    #[test]
    fn test_extract_highlight_terms_empty_query() {
        let idf_map = IDF::new();
        let terms = extract_highlight_terms("", &idf_map, 1.5);
        assert!(terms.is_empty());
    }

    #[test]
    fn test_extract_highlight_terms_no_matches() {
        let idf_map = IDF::new();
        let terms = extract_highlight_terms("unknown words here", &idf_map, 1.5);
        assert!(terms.is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p data_access -- tests::test_extract_highlight_terms 2>&1 | tail -20`
Expected: FAIL — `extract_highlight_terms` does not exist yet.

- [ ] **Step 3: Write the implementation**

Add this function in `data_access/src/embed/mod.rs` after the existing `embed_sparse` function (after line 54):

```rust
pub fn extract_highlight_terms(query: &str, idf_map: &IDF, min_base_idf: f32) -> Vec<String> {
    let (ngram1, ngram2, ngram3) = process_text_to_words(query);

    let mut terms: Vec<(String, f32)> = ngram1
        .iter()
        .chain(ngram2.iter())
        .chain(ngram3.iter())
        .filter_map(|word| {
            let (_, weighted_idf) = idf_map.get(word)?;
            let ngram_weight = (word.matches(' ').count() as f32 + 1.0).min(3.0);
            let base_idf = weighted_idf / ngram_weight;
            if base_idf >= min_base_idf {
                Some((word.clone(), *weighted_idf))
            } else {
                None
            }
        })
        .collect();

    terms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    terms.dedup_by(|a, b| a.0 == b.0);
    terms.into_iter().map(|(term, _)| term).collect()
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p data_access -- tests::test_extract_highlight_terms 2>&1 | tail -20`
Expected: all 4 tests PASS.

- [ ] **Step 5: Commit**

```bash
git add data_access/src/embed/mod.rs
git commit -m "feat: add extract_highlight_terms() with IDF threshold filtering"
```

---

### Task 2: Add `highlight_terms` field to `SearchResult`

**Files:**
- Modify: `data_structures/src/intermediate/search.rs:7-13`

- [ ] **Step 1: Add the field**

In `data_structures/src/intermediate/search.rs`, add `highlight_terms` to the `SearchResult` struct:

```rust
#[derive(Debug, Default, Clone, Serialize, JsonSchema)]
pub struct SearchResult {
    pub query: String,
    pub filter: Option<Filter>,
    pub chunks: Vec<SearchResultChunk>,
    pub suggestions: SearchSuggestions,
    pub highlight_terms: Vec<String>,
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p data_structures 2>&1 | tail -10`
Expected: compiles successfully. (`Default` derives empty `Vec` automatically, `Serialize`/`JsonSchema` handle `Vec<String>` natively.)

- [ ] **Step 3: Commit**

```bash
git add data_structures/src/intermediate/search.rs
git commit -m "feat: add highlight_terms field to SearchResult"
```

---

### Task 3: Add config field and wire through `Searcher`

**Files:**
- Modify: `data_processing/src/config.rs:1-7`
- Modify: `data_processing/src/search.rs:45-71` (Searcher struct + new), `data_processing/src/search.rs:157-166` (search return)
- Modify: `config.toml.example:35-37`

- [ ] **Step 1: Add config field**

In `data_processing/src/config.rs`, add the optional threshold field:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DataProcessingConfig {
    pub tdps_markdown_root: String,
    pub tdps_pdf_root: String,
    pub highlight_idf_threshold: Option<f32>,
}
```

- [ ] **Step 2: Add threshold to `Searcher` struct and constructor**

In `data_processing/src/search.rs`, add the import and field. The `Searcher` struct (line 45) becomes:

```rust
pub struct Searcher {
    pub embed_client: Arc<dyn EmbedClient + Send + Sync>,
    pub vector_client: Arc<dyn VectorClient + Send + Sync>,
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub idf_map: Arc<IDF>,
    pub teams: Vec<String>,
    pub leagues: Vec<String>,
    pub highlight_idf_threshold: f32,
}
```

Update `Searcher::new()` (line 55) to accept the threshold:

```rust
    pub fn new(
        embed_client: Arc<dyn EmbedClient + Send + Sync>,
        vector_client: Arc<dyn VectorClient + Send + Sync>,
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        idf_map: Arc<IDF>,
        teams: Vec<String>,
        leagues: Vec<String>,
        highlight_idf_threshold: f32,
    ) -> Self {
        Self {
            embed_client,
            vector_client,
            metadata_client,
            idf_map,
            teams,
            leagues,
            highlight_idf_threshold,
        }
    }
```

- [ ] **Step 3: Call `extract_highlight_terms` in `search()` and set the field**

Add the import at the top of `data_processing/src/search.rs`:

```rust
use data_access::embed::extract_highlight_terms;
```

In the `search()` method, before building the `Ok(SearchResult { ... })` return (line 157), add:

```rust
        let highlight_terms = extract_highlight_terms(query_trim, &self.idf_map, self.highlight_idf_threshold);
```

And update the return value to include the new field:

```rust
        Ok(SearchResult {
            query,
            filter,
            chunks,
            suggestions: SearchSuggestions {
                teams: team_suggestions,
                leagues: league_suggestions,
            },
            highlight_terms,
        })
```

- [ ] **Step 4: Update config.toml.example**

Add after the `tdps_pdf_root` line in `config.toml.example` (around line 37):

```toml
# highlight_idf_threshold = 1.5  # optional: base IDF below which query terms are not highlighted (default: 1.5)
```

- [ ] **Step 5: Verify it compiles (will fail — callers need updating)**

Run: `cargo check 2>&1 | tail -20`
Expected: compile errors in `web/src/main.rs`, `mcp/src/main.rs`, `tools/src/bin/smoke_test.rs`, `tools/src/bin/search_by_sentence.rs` — they call `Searcher::new()` without the new argument. This is expected and fixed in the next step.

- [ ] **Step 6: Update all `Searcher::new()` call sites**

In `web/src/main.rs` (line 62), update the `Searcher::new()` call:

```rust
    let searcher = Searcher::new(
        embed_client.clone(),
        vector_client.clone(),
        metadata_client.clone(),
        Arc::new(idf_map),
        teams,
        leagues,
        config.data_processing.highlight_idf_threshold.unwrap_or(1.5),
    );
```

In `mcp/src/main.rs` (line 61), same change:

```rust
    let searcher = Searcher::new(
        embed_client.clone(),
        vector_client.clone(),
        metadata_client.clone(),
        Arc::new(idf_map),
        teams,
        leagues,
        config.data_processing.highlight_idf_threshold.unwrap_or(1.5),
    );
```

In `tools/src/bin/smoke_test.rs` (line 47):

```rust
    let searcher = Searcher::new(
        embed_client,
        vector_client,
        metadata_client.clone(),
        idf_map,
        teams,
        leagues,
        config.data_processing.highlight_idf_threshold.unwrap_or(1.5),
    );
```

In `tools/src/bin/search_by_sentence.rs` (line 102):

```rust
    let searcher = Searcher::new(embed_client, vector_client, metadata_client.clone(), idf_map, teams, leagues, config.data_processing.highlight_idf_threshold.unwrap_or(1.5));
```

- [ ] **Step 7: Verify full compilation and tests**

Run: `cargo check 2>&1 | tail -10`
Expected: compiles successfully.

Run: `cargo test 2>&1 | tail -20`
Expected: all tests pass (including the new `extract_highlight_terms` tests from Task 1).

- [ ] **Step 8: Commit**

```bash
git add data_processing/src/config.rs data_processing/src/search.rs config.toml.example web/src/main.rs mcp/src/main.rs tools/src/bin/smoke_test.rs tools/src/bin/search_by_sentence.rs
git commit -m "feat: wire highlight_idf_threshold config through Searcher to extract_highlight_terms"
```

---

### Task 4: Update frontend to use `highlight_terms`

**Files:**
- Modify: `frontend/src/lib/types.ts:46-51`
- Modify: `frontend/src/routes/(browse)/search/+page.svelte:85-91`
- Modify: `frontend/src/lib/components/PaperGroup.svelte:1-11` (props), `PaperGroup.svelte:57-76` (ChunkResult calls)
- Modify: `frontend/src/lib/components/ChunkResult.svelte:1-64`

- [ ] **Step 1: Add `highlight_terms` to TypeScript interface**

In `frontend/src/lib/types.ts`, update the `SearchResult` interface (line 46):

```typescript
export interface SearchResult {
	query: string;
	filter: Filter | null;
	chunks: SearchResultChunk[];
	suggestions: SearchSuggestions;
	highlight_terms: string[];
}
```

- [ ] **Step 2: Pass `highlightTerms` from `+page.svelte` to `PaperGroup`**

In `frontend/src/routes/(browse)/search/+page.svelte`, update the `PaperGroup` usage (line 86):

```svelte
					<PaperGroup
						paperId={group.paperId}
						chunks={group.chunks}
						highlightTerms={data.searchResult.highlight_terms}
					/>
```

- [ ] **Step 3: Update `PaperGroup.svelte` props and forwarding**

Replace the props interface and destructuring (lines 1-11):

```svelte
<script lang="ts">
	import type { SearchResultChunk } from '$lib/types';
	import ChunkResult from './ChunkResult.svelte';

	interface Props {
		paperId: string;
		chunks: SearchResultChunk[];
		highlightTerms: string[];
	}

	let { paperId, chunks, highlightTerms }: Props = $props();
```

Update all `ChunkResult` usages. The first block (lines 57-66):

```svelte
			{#each topChunks as chunk}
				<ChunkResult
					text={chunk.text}
					{highlightTerms}
					score={chunk.score}
					breadcrumbs={chunk.breadcrumbs}
					title={chunk.title}
					{paperId}
				/>
			{/each}
```

The expanded block (lines 70-76):

```svelte
					{#each remainingChunks as chunk}
						<ChunkResult
							text={chunk.text}
							{highlightTerms}
							score={chunk.score}
							{paperId}
						/>
					{/each}
```

- [ ] **Step 4: Rewrite `ChunkResult.svelte` to use `highlightTerms`**

Replace the `<script>` block in `frontend/src/lib/components/ChunkResult.svelte` (lines 1-83):

```svelte
<script lang="ts">
	import type { BreadcrumbEntry } from '$lib/types';
	import { slugifyHeading } from '$lib/markdown';

	interface Props {
		text: string;
		highlightTerms: string[];
		score: number;
		breadcrumbs?: BreadcrumbEntry[];
		title?: string;
		paperId: string;
	}

	let { text, highlightTerms, score, breadcrumbs, title, paperId }: Props = $props();

	function highlightText(text: string, terms: string[]): string {
		if (terms.length === 0) return text;

		// Sort by length descending so longer terms (bigrams/trigrams) match
		// before their constituent unigrams in the regex alternation
		const sorted = [...terms].sort((a, b) => b.length - a.length);

		// Phase 1: collect all match ranges against the original plain text
		const pattern = sorted.map(escapeRegex).join('|');
		const regex = new RegExp(pattern, 'gi');
		const matches: { start: number; end: number }[] = [];
		let m: RegExpExecArray | null;
		while ((m = regex.exec(text)) !== null) {
			matches.push({ start: m.index, end: m.index + m[0].length });
		}
		if (matches.length === 0) return text;

		// Merge overlapping/adjacent ranges
		matches.sort((a, b) => a.start - b.start);
		const merged = [matches[0]];
		for (let i = 1; i < matches.length; i++) {
			const last = merged[merged.length - 1];
			if (matches[i].start <= last.end) {
				last.end = Math.max(last.end, matches[i].end);
			} else {
				merged.push(matches[i]);
			}
		}

		// Phase 2: build result by slicing original text and wrapping matched ranges
		const tag = '<mark class="bg-yellow-200 dark:bg-yellow-700/60 dark:text-yellow-100">';
		let result = '';
		let cursor = 0;
		for (const { start, end } of merged) {
			result += text.slice(cursor, start) + tag + text.slice(start, end) + '</mark>';
			cursor = end;
		}
		result += text.slice(cursor);
		return result;
	}

	function escapeRegex(str: string): string {
		return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
	}

	const highlighted = $derived(highlightText(text, highlightTerms));

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
```

The HTML template (lines 84 onward) stays unchanged.

- [ ] **Step 5: Verify frontend builds**

Run: `cd frontend && npx svelte-check --threshold error 2>&1 | tail -5`
Expected: 0 errors. (The pre-existing `PaperGroup.svelte` error about missing `paperId` on expanded chunks should also be fixed now since we're passing `{paperId}` to all `ChunkResult` instances.)

- [ ] **Step 6: Commit**

```bash
git add frontend/src/lib/types.ts frontend/src/routes/\(browse\)/search/+page.svelte frontend/src/lib/components/PaperGroup.svelte frontend/src/lib/components/ChunkResult.svelte
git commit -m "feat: frontend uses backend highlight_terms instead of query splitting"
```

---

### Task 5: End-to-end verification

- [ ] **Step 1: Run full backend test suite**

Run: `cargo test 2>&1 | tail -20`
Expected: all tests pass.

- [ ] **Step 2: Build frontend for production**

Run: `cd frontend && npm run build 2>&1 | tail -10`
Expected: builds successfully with no errors.

- [ ] **Step 3: Manual verification (optional — requires running services)**

If Qdrant and the web server are available:

1. Build and start: `docker compose up web --build -d`
2. Open: `https://web.emielsteerneman.nl/search?q=I+am+looking+for+the+designs+of+a+solenoid+winder`
3. Verify: only meaningful terms like "designs", "solenoid", "winder" are highlighted — not "I", "am", "for", "the", "of", "a"
4. Verify: no raw HTML tags visible in output
5. Verify: bigrams like "solenoid winder" highlight as a unit where they appear together
