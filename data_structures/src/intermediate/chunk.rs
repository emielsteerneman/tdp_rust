use serde::Serialize;

use crate::file::{League, TeamName};

#[derive(Clone, Debug, Default, Serialize)]
pub struct ChunkMetadata {
    pub team: Option<String>,
    pub league: Option<String>,
    pub year: Option<i32>,
    pub section: Option<String>,
    pub has_figures: Option<bool>,
    pub has_equations: Option<bool>,
    pub source_id: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct Chunk {
    pub embedding: Vec<f32>,
    // Filter
    pub league_year_team_idx: String,
    pub league: League,
    pub year: u32,
    pub team: TeamName,
    // Reconstruct
    paragraph_sequence_id: u32,
    chunk_sequence_id: u32,
    idx_begin: u32,
    idx_end: u32,
    pub text: String,
}

// impl Chunk {
//     pub fn to_uuid(&self) -> Uuid {
//         static ZERO_NAMESPACE: Uuid = Uuid::from_bytes([0u8; 16]);
//         let s = self.league_year_team_idx;
//         Uuid::new_v5(&ZERO_NAMESPACE, s.as_bytes())
//     }
// }

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nChunk {{ start: {}, end: {}, text_length: {}, text: '{}' }}",
            self.idx_begin,
            self.idx_end,
            self.text.len(),
            self.text
        )
    }
}
