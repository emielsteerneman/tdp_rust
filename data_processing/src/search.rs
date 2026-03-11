use std::collections::HashMap;
use std::sync::Arc;

use data_access::embed::EmbedClient;
use data_access::metadata::MetadataClient;
use data_access::vector::VectorClient;
use data_structures::{
    IDF,
    content::TocEntry,
    embed_type::EmbedType,
    filter::Filter,
    intermediate::{
        BreadcrumbEntry, EnrichedChunk, EnrichedSearchResult, ScoredChunk, SearchResult,
        SearchSuggestions,
    },
};
use tracing::{info, warn};

use crate::text::match_terms;

/// Compute the breadcrumb path for a given content_seq within a table of contents.
fn compute_breadcrumbs(toc: &[TocEntry], content_seq: u32) -> Vec<BreadcrumbEntry> {
    let Some(target_idx) = toc.iter().position(|e| e.content_seq == content_seq) else {
        return Vec::new();
    };
    let target_depth = toc[target_idx].depth;

    let mut crumbs = Vec::new();
    let mut needed_depth = target_depth;

    for entry in toc[..target_idx].iter().rev() {
        if entry.depth < needed_depth {
            crumbs.push(BreadcrumbEntry {
                content_seq: entry.content_seq,
                title: entry.title.clone(),
            });
            needed_depth = entry.depth;
            if needed_depth == 0 {
                break;
            }
        }
    }

    crumbs.reverse();
    crumbs
}

pub struct Searcher {
    pub embed_client: Arc<dyn EmbedClient + Send + Sync>,
    pub vector_client: Arc<dyn VectorClient + Send + Sync>,
    pub metadata_client: Arc<dyn MetadataClient + Send + Sync>,
    pub idf_map: Arc<IDF>,
    pub teams: Vec<String>,
    pub leagues: Vec<String>,
}

impl Searcher {
    pub fn new(
        embed_client: Arc<dyn EmbedClient + Send + Sync>,
        vector_client: Arc<dyn VectorClient + Send + Sync>,
        metadata_client: Arc<dyn MetadataClient + Send + Sync>,
        idf_map: Arc<IDF>,
        teams: Vec<String>,
        leagues: Vec<String>,
    ) -> Self {
        Self {
            embed_client,
            vector_client,
            metadata_client,
            idf_map,
            teams,
            leagues,
        }
    }

    /// Plain search without breadcrumb enrichment (backwards-compatible).
    pub async fn search(
        &self,
        query: String,
        limit: Option<u64>,
        filter: Option<Filter>,
        search_type: EmbedType,
    ) -> anyhow::Result<SearchResult> {
        let enriched = self
            .search_enriched(query, limit, filter, search_type)
            .await?;

        Ok(SearchResult {
            query: enriched.query,
            filter: enriched.filter,
            chunks: enriched
                .chunks
                .into_iter()
                .map(|ec| ScoredChunk {
                    chunk: ec.chunk,
                    score: ec.score,
                })
                .collect(),
            suggestions: enriched.suggestions,
        })
    }

    /// Search with breadcrumb-enriched results.
    pub async fn search_enriched(
        &self,
        query: String,
        limit: Option<u64>,
        filter: Option<Filter>,
        search_type: EmbedType,
    ) -> anyhow::Result<EnrichedSearchResult> {
        info!("Search n={limit:?} type={search_type:?} filter={filter:?}");
        info!("Query : {query}");

        let limit = limit.unwrap_or(15);
        let query_trim = query.trim();
        if query_trim.is_empty() {
            return Ok(EnrichedSearchResult {
                query,
                filter,
                chunks: Vec::new(),
                suggestions: SearchSuggestions::default(),
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

        // Collect unique lytis to batch-load ToCs
        let unique_lytis: Vec<String> = {
            let mut seen = std::collections::HashSet::new();
            results
                .iter()
                .filter_map(|(chunk, _)| {
                    if seen.insert(chunk.league_year_team_idx.clone()) {
                        Some(chunk.league_year_team_idx.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };

        // Load ToCs for breadcrumb computation
        let mut toc_cache: HashMap<String, Vec<TocEntry>> = HashMap::new();
        for lyti in unique_lytis {
            match self.metadata_client.load_toc(lyti.clone()).await {
                Ok(toc) => {
                    toc_cache.insert(lyti, toc);
                }
                Err(e) => {
                    warn!("Failed to load ToC for {}: {}", lyti, e);
                }
            }
        }

        let team_suggestions = match_terms(self.teams.clone(), query_trim.to_string(), Some(0.8));
        let league_suggestions =
            match_terms(self.leagues.clone(), query_trim.to_string(), Some(0.8));

        let chunks = results
            .into_iter()
            .map(|(chunk, score)| {
                let breadcrumbs = toc_cache
                    .get(&chunk.league_year_team_idx)
                    .map(|toc| compute_breadcrumbs(toc, chunk.content_seq))
                    .unwrap_or_default();
                EnrichedChunk {
                    chunk: chunk.into(),
                    score,
                    breadcrumbs,
                }
            })
            .collect();

        Ok(EnrichedSearchResult {
            query,
            filter,
            chunks,
            suggestions: SearchSuggestions {
                teams: team_suggestions,
                leagues: league_suggestions,
            },
        })
    }
}
