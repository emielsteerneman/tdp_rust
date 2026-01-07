use data_access::embed::EmbedClient;
use data_structures::{intermediate::Chunk, paper::TDP};
use tracing::{info, instrument};

use super::create_sentence_chunks;

#[instrument("data_processing", skip_all)]
pub async fn tdp_to_chunks(tdp: &TDP, embed_client: Option<&dyn EmbedClient>) -> Vec<Chunk> {
    let mut chunks = Vec::<Chunk>::new();

    for (i_paragraph, paragraph) in tdp.structure.paragraphs.iter().enumerate() {
        info!("Processing paragraph: {}", paragraph.title.raw);
        let raw_chunks = create_sentence_chunks(paragraph.sentences.clone(), 500, 100);

        for raw_chunk in raw_chunks {
            let chunk = raw_chunk
                .into_chunk(embed_client, tdp.name.clone(), i_paragraph)
                .await;

            chunks.push(chunk);
        }
    }

    chunks
}
