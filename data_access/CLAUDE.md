## Purpose
Trait-defined storage abstractions and their implementations. All external systems (Qdrant, SQLite, OpenAI) are behind async traits for testability and swappability.

## Traits and Implementations
- `EmbedClient` — generates embeddings. Methods: `embed_string`, `embed_strings`, `embed_sparse` (default impl). Impls: `OpenAIClient` (API, batches of 100), `FastembedClient` (local ONNX inference).
- `VectorClient` — stores/searches chunks. Impl: `QdrantClient` (gRPC, hybrid search with RRF fusion).
- `MetadataClient` — paper metadata, IDF storage, TOC, references (14 methods). Impl: `SqliteClient`. Has `#[automock]` for testing.
- `RegistryClient` — team and league metadata key-value store with HMAC auth. Methods: `get_team_metadata`, `set_team_metadata`, `verify_code`, `generate_team_code`, `get_league_metadata`, `set_league_metadata`. Impl: `SqliteRegistryClient`.

## Patterns
- Two async trait styles coexist: `#[async_trait]` (VectorClient) and manual `Pin<Box<dyn Future>>` (others, for lifetime control).
- Thread safety: `Arc<Mutex<Connection>>` for SQLite, `Send + Sync` bounds on traits.
- Config structs live in `config.rs` — deserialized from TOML by the `configuration` crate.

## Gotchas
- Qdrant collection uses named vectors: `"dense"` and `"sparse"`. Hybrid search uses Reciprocal Rank Fusion.
- SQLite clients enable WAL mode on init for concurrent reads.
- OpenAI embed client sleeps 1s between batches for rate limiting.
