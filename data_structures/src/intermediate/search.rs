use serde::Serialize;
use schemars::JsonSchema;
use crate::file::{League, TeamName};
use crate::filter::Filter;
use crate::intermediate::Chunk;

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SearchResult {
    pub query: String,
    pub filter: Option<Filter>,
    pub chunks: Vec<ScoredChunk>,
    pub suggestions: SearchSuggestions,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct ScoredChunk {
    pub chunk: SearchResultChunk,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct SearchResultChunk {
    pub league_year_team_idx: String,
    pub league: League,
    pub year: u32,
    pub team: TeamName,
    pub paragraph_sequence_id: u32,
    pub chunk_sequence_id: u32,
    pub idx_begin: u32,
    pub idx_end: u32,
    pub text: String,
}

impl From<Chunk> for SearchResultChunk {
    fn from(chunk: Chunk) -> Self {
        Self {
            league_year_team_idx: chunk.league_year_team_idx,
            league: chunk.league,
            year: chunk.year,
            team: chunk.team,
            paragraph_sequence_id: chunk.paragraph_sequence_id,
            chunk_sequence_id: chunk.chunk_sequence_id,
            idx_begin: chunk.idx_begin,
            idx_end: chunk.idx_end,
            text: chunk.text,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default, JsonSchema)]
pub struct SearchSuggestions {
    pub teams: Vec<String>,
    pub leagues: Vec<String>,
}