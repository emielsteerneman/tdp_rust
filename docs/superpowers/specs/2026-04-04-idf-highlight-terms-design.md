# IDF-Based Highlight Terms for Search Results

## Problem

The frontend highlights search query terms in result text by naively splitting the raw query string into words and wrapping each match in `<mark>` tags. This causes two bugs:

1. **HTML corruption**: The old code did N sequential `.replace()` calls — one per word. Each pass scanned the *output* of the previous pass, matching inside injected `<mark>` tag attributes (e.g., "i" matching inside `bg-yellow-200`), producing broken HTML visible as raw tags in the UI.

2. **Noise**: Stop words like "I", "a", "of", "the" get highlighted everywhere, obscuring the actual matches.

The HTML corruption was fixed by switching to a two-phase approach (collect match positions on plain text, then insert tags). But the noise problem remains — the frontend has no way to know which query terms are meaningful.

## Solution

The backend already has the IDF (Inverse Document Frequency) map loaded in `Searcher`. Terms with high IDF scores are rare and significant; terms with low or absent IDF scores are common/noise. We extract highlight terms from the query using the IDF map and return them alongside search results.

## Design

### Config: Highlight threshold

Add an optional `highlight_idf_threshold` field to `DataProcessingConfig` in `data_processing/src/config.rs`:

```rust
pub struct DataProcessingConfig {
    pub tdps_markdown_root: String,
    pub tdps_pdf_root: String,
    pub highlight_idf_threshold: Option<f32>,  // default 1.5
}
```

In `config.toml`:
```toml
[data_processing]
tdps_markdown_root = "/path/to/tdps_markdown/"
tdps_pdf_root = "/path/to/tdps_pdf/"
highlight_idf_threshold = 1.5  # optional, base IDF below which terms are not highlighted
```

The threshold applies to the **base IDF** (before n-gram weighting). The IDF formula `log10((N+1)/(DF(t)+1)) + 1` produces:
- ~1.0 for terms in every document (noise)
- ~1.5 for terms in ~30% of documents
- ~2.0 for terms in ~10% of documents
- ~3.0+ for rare, specific terms

A default of **1.5** filters out terms appearing in more than ~30% of all chunks. The IDF map stores `weighted_idf = base_idf * ngram_weight` (weights: 1x unigrams, 2x bigrams, 3x trigrams). To recover the base IDF, we divide by the n-gram weight, inferred from space count in the term string (0 spaces = unigram/1x, 1 space = bigram/2x, 2 spaces = trigram/3x).

### Backend: `extract_highlight_terms`

New function in `data_access/src/embed/mod.rs`, alongside the existing `embed_sparse`:

```rust
pub fn extract_highlight_terms(query: &str, idf_map: &IDF, min_base_idf: f32) -> Vec<String> {
    let (ngram1, ngram2, ngram3) = process_text_to_words(query);
    let mut terms: Vec<(String, f32)> = ngram1.iter()
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
    terms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    terms.dedup_by(|a, b| a.0 == b.0);
    terms.into_iter().map(|(term, _)| term).collect()
}
```

- Runs `process_text_to_words()` on the query (same tokenization used for sparse embeddings)
- Filters to terms present in the IDF map whose **base IDF** meets the threshold
- Recovers base IDF by dividing weighted IDF by n-gram weight (inferred from space count)
- Returns terms sorted by weighted IDF descending (most significant first)
- Includes bigrams ("solenoid winder") and trigrams naturally — they're in the IDF map with higher weights

### Data types: New field on `SearchResult`

Add `highlight_terms: Vec<String>` to `SearchResult` in `data_structures/src/intermediate/search.rs`:

```rust
pub struct SearchResult {
    pub query: String,
    pub filter: Option<Filter>,
    pub chunks: Vec<SearchResultChunk>,
    pub suggestions: SearchSuggestions,
    pub highlight_terms: Vec<String>,
}
```

This is query-level, not per-chunk — all chunks from the same search share the same highlight terms.

### Integration: `Searcher::search()`

`Searcher` gains a `highlight_idf_threshold: f32` field, set from config (with default 1.5). In `search()`, call `extract_highlight_terms` with the threshold:

```rust
let highlight_terms = extract_highlight_terms(query_trim, &self.idf_map, self.highlight_idf_threshold);
// ...
Ok(SearchResult {
    query,
    filter,
    chunks,
    suggestions: ...,
    highlight_terms,
})
```

The threshold flows: `config.toml` → `DataProcessingConfig` → `Searcher::new()` → `extract_highlight_terms()`.

### Frontend types

Mirror in `frontend/src/lib/types.ts`:

```typescript
export interface SearchResult {
    query: string;
    filter: Filter | null;
    chunks: SearchResultChunk[];
    suggestions: SearchSuggestions;
    highlight_terms: string[];
}
```

### Frontend components

**`+page.svelte`**: Pass `highlight_terms` down instead of `query` for highlighting purposes.

**`PaperGroup.svelte`**: Accept `highlightTerms: string[]` prop, pass to `ChunkResult`.

**`ChunkResult.svelte`**: Accept `highlightTerms: string[]` instead of `query: string`. The existing two-phase `highlightText()` function uses these terms directly as regex alternation — no splitting, no stop word filtering needed.

**N-gram ordering in regex**: Before building the regex alternation, sort terms by **length descending**. Regex alternation is left-to-right greedy — `(computer vision|computer)` matches the bigram first, but `(computer|computer vision)` would match only the unigram at that position, leaving "vision" unhighlighted. Length-descending sorting ensures longer (more specific) terms always get priority. The merge step provides a safety net (overlapping ranges like `[5,13]` and `[5,20]` merge to `[5,20]`), but correct ordering avoids the problem entirely.

### MCP: No changes

The MCP server maps `SearchResult` into its own `CompactSearchResult`, cherry-picking only `query`, `results`, and `suggestions`. The new `highlight_terms` field is silently ignored. AI clients don't render HTML highlighting.

## Files Changed

| File | Change |
|------|--------|
| `data_processing/src/config.rs` | Add optional `highlight_idf_threshold` field (default 1.5) |
| `data_access/src/embed/mod.rs` | Add `extract_highlight_terms()` function next to `embed_sparse()` |
| `data_structures/src/intermediate/search.rs` | Add `highlight_terms: Vec<String>` to `SearchResult` |
| `data_processing/src/search.rs` | Add threshold to `Searcher`, call `extract_highlight_terms()` |
| `frontend/src/lib/types.ts` | Add `highlight_terms: string[]` to `SearchResult` |
| `frontend/src/routes/(browse)/search/+page.svelte` | Pass `highlight_terms` to `PaperGroup` |
| `frontend/src/lib/components/PaperGroup.svelte` | Accept and forward `highlightTerms` |
| `frontend/src/lib/components/ChunkResult.svelte` | Use `highlightTerms` array directly instead of splitting query |
| `config.toml.example` | Add commented `highlight_idf_threshold` example |
