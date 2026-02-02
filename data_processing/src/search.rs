use data_access::embed::EmbedClient;
use data_access::vector::VectorClient;
use data_structures::{
    IDF,
    embed_type::EmbedType,
    filter::Filter,
    intermediate::{ScoredChunk, SearchResult, SearchSuggestions},
};
use std::sync::Arc;
use tracing::{info, instrument};

use crate::text::match_terms;

pub struct Searcher {
    pub embed_client: Arc<dyn EmbedClient + Send + Sync>,
    pub vector_client: Arc<dyn VectorClient + Send + Sync>,
    pub idf_map: Arc<IDF>,
    pub teams: Vec<String>,
    pub leagues: Vec<String>,
}

impl Searcher {
    pub fn new(
        embed_client: Arc<dyn EmbedClient + Send + Sync>,
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

    pub async fn search(
        &self,
        query: String,
        limit: Option<u64>,
        filter: Option<Filter>,
        search_type: EmbedType,
    ) -> anyhow::Result<SearchResult> {
        info!("\nSearch n={limit:?} type={search_type:?} filter={filter:?}");
        info!("Query : {query}");

        let limit = limit.unwrap_or(15);
        let query_trim = query.trim();
        if query_trim.is_empty() {
            return Ok(SearchResult {
                query: query,
                filter,
                ..Default::default()
            });
        }

        let dense = if matches!(search_type, EmbedType::DENSE | EmbedType::HYBRID) {
            Some(self.embed_client.embed_string(query_trim).await?)
        } else {
            None
        };

        let sparse = if matches!(search_type, EmbedType::SPARSE | EmbedType::HYBRID) {
            Some(self.embed_client.embed_sparse(query_trim, &self.idf_map))
        } else {
            None
        };

        let results = self
            .vector_client
            .search_chunks(dense, sparse, limit, filter.clone())
            .await?;

        let team_suggestions = match_terms(self.teams.clone(), query_trim.to_string());
        let league_suggestions = match_terms(self.leagues.clone(), query_trim.to_string());

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
