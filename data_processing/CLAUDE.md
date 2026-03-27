## Purpose
Transforms raw markdown TDPs into searchable chunks and orchestrates search queries.

## Pipeline
```
Raw markdown → parse_markdown() → MarkdownTDP
    → tdp_to_chunks() → Vec<Chunk> (text split at 1500 chars, 200 char overlap)
    → embed_chunks() → Vec<Chunk> with dense + sparse embeddings
```

## Key Types
- `Searcher` — holds all clients + IDF map + team/league lists. Main entry: `search(query, limit, filter, search_type)`.

## Modules
- `markdown_parser` — state-machine parser for TDP markdown format (title, authors, abstract, paragraphs, tables, images, references).
- `content_chunker` — splits content items into chunks. Text split on `\n\n` boundaries, tables kept whole, images use title as text.
- `embed/` — calls embed client for dense vectors, computes sparse vectors using IDF weights.
- `text/create_idf` — builds IDF from 1/2/3-grams with weighted scoring. Higher n-grams get higher multipliers.
- `text/match_terms` — Jaro-Winkler fuzzy matching for team/league suggestions (threshold 0.8).

## Note
The actual ranking/fusion of dense+sparse results happens in the VectorClient (Qdrant RRF), not here. This crate prepares both embedding types and delegates search to data_access.
