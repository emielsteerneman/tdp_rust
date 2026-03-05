use std::collections::HashMap;

use serde::Serialize;
use uuid::Uuid;

use crate::content::ContentType;
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

#[derive(Clone, Default, Serialize)]
pub struct Chunk {
    pub dense_embedding: Vec<f32>,
    pub sparse_embedding: HashMap<u32, f32>,
    // Filter
    pub league_year_team_idx: String,
    pub league: League,
    pub year: u32,
    pub team: TeamName,
    // Reconstruct
    pub content_seq: u32,
    pub chunk_seq: u32,
    pub content_type: ContentType,
    pub title: String,
    pub image_path: Option<String>,
    pub text: String,
}

impl Chunk {
    pub fn to_uuid(&self) -> Uuid {
        static ZERO_NAMESPACE: Uuid = Uuid::from_bytes([0u8; 16]);
        let s = format!(
            "{}__{}__{}",
            self.league_year_team_idx.clone(),
            self.content_seq,
            self.chunk_seq
        );

        Uuid::new_v5(&ZERO_NAMESPACE, s.as_bytes())
    }
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\nChunk {{ content_seq: {}, chunk_seq: {}, content_type: {}, text_length: {}, text: '{}' }}",
            self.content_seq,
            self.chunk_seq,
            self.content_type.as_str(),
            self.text.len(),
            self.text
        )
    }
}
