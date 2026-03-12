use crate::file::{League, TeamName};
use crate::filter::Filter;
use crate::intermediate::{BreadcrumbEntry, Chunk};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Debug, Default, Clone, Serialize, JsonSchema)]
pub struct SearchResult {
    pub query: String,
    pub filter: Option<Filter>,
    pub chunks: Vec<SearchResultChunk>,
    pub suggestions: SearchSuggestions,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SearchResultChunk {
    pub league_year_team_idx: String,
    pub league: League,
    pub year: u32,
    pub team: TeamName,
    pub content_seq: u32,
    pub chunk_seq: u32,
    pub content_type: String,
    pub title: String,
    pub text: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub breadcrumbs: Vec<BreadcrumbEntry>,
}

impl From<Chunk> for SearchResultChunk {
    fn from(chunk: Chunk) -> Self {
        Self {
            league_year_team_idx: chunk.league_year_team_idx,
            league: chunk.league,
            year: chunk.year,
            team: chunk.team,
            content_seq: chunk.content_seq,
            chunk_seq: chunk.chunk_seq,
            content_type: chunk.content_type.as_str().to_string(),
            title: chunk.title,
            text: chunk.text,
            score: 0.0,
            breadcrumbs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
pub struct SearchSuggestions {
    pub teams: Vec<String>,
    pub leagues: Vec<String>,
}
