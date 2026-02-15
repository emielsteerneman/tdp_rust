Switch from `config-rs` to `figment` ?
Use League and TeamName instead of strings for filters
Read https://qdrant.tech/articles/hybrid-search/
Add exponential backoff retry for OpenAI API rate limits (1M tokens/min) in embed_client


Create a single struct that holds everything related to search. SearchInput.

Create appropriate tools
Metadata:
    Get all teams:
        * Hint
        * Filter: league
    Get all leagues
        * Hint

