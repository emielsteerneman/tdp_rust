# IDF-Based Highlight Terms for Search Results

## Problem

The frontend highlights search query terms in result text by naively splitting the raw query string into words and wrapping each match in `<mark>` tags. This causes two bugs:

1. **HTML corruption**: The old code did N sequential `.replace()` calls — one per word. Each pass scanned the *output* of the previous pass, matching inside injected `<mark>` tag attributes (e.g., "i" matching inside `bg-yellow-200`), producing broken HTML visible as raw tags in the UI.

2. **Noise**: Stop words like "I", "a", "of", "the" get highlighted everywhere, obscuring the actual matches.

The HTML corruption was fixed by switching to a two-phase approach (collect match positions on plain text, then insert tags). But the noise problem remains — the frontend has no way to know which query terms are meaningful.

## Solution

The backend already has the IDF (Inverse Document Frequency) map loaded in `Searcher`. Terms with high IDF scores are rare and significant; terms with low or absent IDF scores are common/noise. We extract highlight terms from the query using the IDF map and return them alongside search results.

## Design

### Backend: `extract_highlight_terms`

New function in `data_access/src/embed/mod.rs`, alongside the existing `embed_sparse`:

```rust
pub fn extract_highlight_terms(query: &str, idf_map: &IDF) -> Vec<String> {
    let (ngram1, ngram2, ngram3) = process_text_to_words(query);
    let mut terms: Vec<(String, f32)> = ngram1.iter()
        .chain(ngram2.iter())
        .chain(ngram3.iter())
        .filter_map(|word| idf_map.get(word).map(|(_, idf)| (word.clone(), *idf)))
        .collect();
    terms.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    terms.dedup_by(|a, b| a.0 == b.0);
    terms.into_iter().map(|(term, _)| term).collect()
}
```

- Runs `process_text_to_words()` on the query (same tokenization used for sparse embeddings)
- Filters to terms present in the IDF map (terms not in the map are too rare or too common to be indexed)
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

In `data_processing/src/search.rs`, call `extract_highlight_terms` and set the field on the returned `SearchResult`:

```rust
let highlight_terms = extract_highlight_terms(query_trim, &self.idf_map);
// ...
Ok(SearchResult {
    query,
    filter,
    chunks,
    suggestions: ...,
    highlight_terms,
})
```

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

### MCP: No changes

The MCP server maps `SearchResult` into its own `CompactSearchResult`, cherry-picking only `query`, `results`, and `suggestions`. The new `highlight_terms` field is silently ignored. AI clients don't render HTML highlighting.

## Files Changed

| File | Change |
|------|--------|
| `data_access/src/embed/mod.rs` | Add `extract_highlight_terms()` function next to `embed_sparse()` |
| `data_structures/src/intermediate/search.rs` | Add `highlight_terms: Vec<String>` to `SearchResult` |
| `data_processing/src/search.rs` | Call `extract_highlight_terms()` in `Searcher::search()` |
| `frontend/src/lib/types.ts` | Add `highlight_terms: string[]` to `SearchResult` |
| `frontend/src/routes/(browse)/search/+page.svelte` | Pass `highlight_terms` to `PaperGroup` |
| `frontend/src/lib/components/PaperGroup.svelte` | Accept and forward `highlightTerms` |
| `frontend/src/lib/components/ChunkResult.svelte` | Use `highlightTerms` array directly instead of splitting query |
