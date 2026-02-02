use std::sync::Arc;
use data_access::vector::VectorClient;
use data_access::embed::{EmbedClient, embed_sparse};
use data_structures::{IDF, intermediate::{ScoredChunk, SearchResult, SearchSuggestions}, filter::Filter};
use crate::utils::match_names;

pub struct Searcher {
    pub embed_client: Option<Arc<dyn EmbedClient + Send + Sync>>,
    pub vector_client: Arc<dyn VectorClient + Send + Sync>,
    pub idf_map: Arc<IDF>,
    pub teams: Vec<String>,
    pub leagues: Vec<String>,
}

impl Searcher {
    pub fn new(
        embed_client: Option<Arc<dyn EmbedClient + Send + Sync>>,
        vector_client: Arc<dyn VectorClient + Send + Sync>,
        idf_map: Arc<IDF>,
        teams: Vec<String>,
        leagues: Vec<String>,
    ) -> Self {
        Self {
            embed_client,
            vector_client,
            idf_map,
            teams,
            leagues,
        }
    }

    pub async fn search(&self, query: String, limit: Option<u64>, filter: Option<Filter>) -> anyhow::Result<SearchResult> {
        let limit = limit.unwrap_or(15);
        let query_trim = query.trim();
        if query_trim.is_empty() {
             return Ok(SearchResult {
                 query: query,
                 filter,
                 chunks: vec![],
                 suggestions: SearchSuggestions::default(),
             });
        }

        let dense = if let Some(client) = &self.embed_client {
            Some(client.embed_string(query_trim).await?)
        } else {
            None
        };

        let sparse = if let Some(client) = &self.embed_client {
            client.embed_sparse(query_trim, &self.idf_map)
        } else {
            embed_sparse(query_trim, &self.idf_map)
        };

        let results = self
            .vector_client
            .search_chunks(dense, Some(sparse), limit, filter.clone())
            .await?;

        let team_suggestions = match_names(self.teams.clone(), query_trim.to_string());
        let league_suggestions = match_names(self.leagues.clone(), query_trim.to_string());

        Ok(SearchResult {
            query: query,
            filter,
            chunks: results
                .into_iter()
                .map(|(chunk, score)| ScoredChunk {
                    chunk: chunk.into(),
                    score,
                })
                .collect(),
            suggestions: SearchSuggestions {
                teams: team_suggestions,
                leagues: league_suggestions,
            },
        })
    }
}
