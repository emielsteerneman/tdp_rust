## Purpose
Loads config from TOML and provides factory functions to construct all clients.

## Config Loading
`AppConfig::load_from_file(path)` uses the `config` crate:
1. Reads TOML file
2. Applies `TDP_*` env var overrides (double underscore `__` = path separator, e.g. `TDP_DATA_ACCESS__EMBED__OPENAI__API_KEY`)

Top-level struct: `AppConfig { data_access, data_processing, event_processing: Option<...>, website_url: Option<String> }`. Note: `event_processing` is optional.

## Factory Functions (in `helpers.rs`)
- `load_any_embed_client()` → `Arc<dyn EmbedClient>` (picks OpenAI or FastEmbed from config)
- `load_any_vector_client()` → `Arc<dyn VectorClient>` (async, Qdrant)
- `load_any_metadata_client()` → `Arc<dyn MetadataClient>` (SQLite)
- `build_registry_client()` → `Option<Arc<dyn RegistryClient>>` (optional)
- `build_event_dispatcher()` → `Arc<EventDispatcher>` (registers configured listeners)

Always use these helpers instead of constructing clients directly.
