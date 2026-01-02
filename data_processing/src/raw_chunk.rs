use data_access::embed::EmbedClient;
use data_structures::{file::TDPName, intermediate::Chunk, paper::Text};

#[derive(Clone, Debug)]
pub struct RawChunk {
    pub sentences: Vec<Text>,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

impl RawChunk {
    pub async fn into_chunk(self, embed_client: &dyn EmbedClient, tdp_name: TDPName) -> Chunk {
        // TODO finish chunk
        let embedding = embed_client.embed_string(&self.text).await.unwrap();

        Chunk {
            sentences: self.sentences,
            start: self.start,
            end: self.end,
            text: self.text,
        }
    }
}
