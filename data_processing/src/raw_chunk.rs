use std::collections::HashMap;

use data_structures::{
    content::ContentType, file::TDPName, intermediate::Chunk, paper::Text,
};

#[derive(Clone, Debug)]
pub struct RawChunk {
    pub chunk_sequence_id: u32,
    pub sentences: Vec<Text>,
    pub idx_begin: u32,
    pub idx_end: u32,
    pub text: String,
}

impl RawChunk {
    pub fn into_chunk(
        self,
        embedding: Option<Vec<f32>>,
        sparse_embedding: Option<HashMap<u32, f32>>,
        tdp_name: TDPName,
        content_seq: usize,
        content_type: ContentType,
        title: String,
        image_path: Option<String>,
    ) -> Chunk {
        Chunk {
            dense_embedding: embedding.unwrap_or_default(),
            sparse_embedding: sparse_embedding.unwrap_or_default(),
            league_year_team_idx: tdp_name.get_filename(),
            league: tdp_name.league,
            year: tdp_name.year,
            team: tdp_name.team_name,
            content_seq: content_seq as u32,
            chunk_seq: self.chunk_sequence_id,
            content_type,
            title,
            image_path,
            text: self.text,
        }
    }
}
