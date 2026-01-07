use data_access::embed::EmbedClient;
use data_structures::{file::TDPName, intermediate::Chunk, paper::Text};

#[derive(Clone, Debug)]
pub struct RawChunk {
    pub chunk_sequence_id: u32,
    pub sentences: Vec<Text>,
    pub idx_begin: u32,
    pub idx_end: u32,
    pub text: String,
}

impl RawChunk {
    pub async fn into_chunk(
        self,
        embed_client: Option<&dyn EmbedClient>,
        tdp_name: TDPName,
        paragraph_sequence_id: usize,
    ) -> Chunk {
        let embedding = if let Some(embed_client) = embed_client {
            embed_client.embed_string(&self.text).await.unwrap()
        } else {
            vec![]
        };

        Chunk {
            embedding,
            league_year_team_idx: tdp_name.get_filename(),
            league: tdp_name.league.name,
            year: tdp_name.year,
            team: tdp_name.team_name.name,
            paragraph_sequence_id: paragraph_sequence_id as u32,
            chunk_sequence_id: self.chunk_sequence_id,
            idx_begin: self.idx_begin,
            idx_end: self.idx_end,
            text: self.text,
        }
    }
}
