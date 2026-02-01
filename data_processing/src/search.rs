use crate::utils::{embed_sparse, match_names};
use data_access::vector::VectorClient;
use data_structures::{
    IDF,
    filter::Filter,
    intermediate::{ScoredChunk, SearchResult, SearchSuggestions},
};
use std::sync::Arc;

pub struct Searcher {
    pub vector_client: Arc<dyn VectorClient + Send + Sync>,
    pub idf_map: Arc<IDF>,
    pub teams: Vec<String>,
    pub leagues: Vec<String>,
}

impl Searcher {
    pub fn new(
        vector_client: Arc<dyn VectorClient + Send + Sync>,
        idf_map: Arc<IDF>,
        teams: Vec<String>,
        leagues: Vec<String>,
    ) -> Self {
        Self {
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
    ) -> anyhow::Result<SearchResult> {
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

        let sparse = embed_sparse(query_trim, &self.idf_map);

        let results = self
            .vector_client
            .search_chunks(None, Some(sparse), limit, filter.clone())
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
