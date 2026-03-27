## Purpose
Trait-defined storage abstractions and their implementations. All external systems (Qdrant, SQLite, OpenAI) are behind async traits for testability and swappability.

## Traits and Implementations
- `EmbedClient` — generates embeddings. Impls: `OpenAIClient` (API, batches of 100), `FastembedClient` (local ONNX inference).
- `VectorClient` — stores/searches chunks. Impl: `QdrantClient` (gRPC, hybrid search with RRF fusion).
- `MetadataClient` — paper metadata, IDF storage, TOC. Impl: `SqliteClient`. Has `#[automock]` for testing.
- `TeamRegistryClient` — team metadata key-value store with HMAC auth. Impl: `TeamsSqliteClient`.

## Patterns
- Two async trait styles coexist: `#[async_trait]` (VectorClient) and manual `Pin<Box<dyn Future>>` (others, for lifetime control).
- Thread safety: `Arc<Mutex<Connection>>` for SQLite, `Send + Sync` bounds on traits.
- Config structs live in `config.rs` — deserialized from TOML by the `configuration` crate.

## Gotchas
- Qdrant collection uses named vectors: `"dense"` and `"sparse"`. Hybrid search uses Reciprocal Rank Fusion.
- SQLite clients enable WAL mode on init for concurrent reads.
- OpenAI embed client sleeps 1s between batches for rate limiting.
