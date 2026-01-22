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

        let texts = raw_chunks
            .iter()
            .map(|chunk| chunk.text.clone())
            .collect::<Vec<String>>();

        let texts_ref = texts.iter().map(|s| s.as_str()).collect::<Vec<&str>>();

        let embeddings: Vec<Option<Vec<f32>>> = if let Some(embed_client) = embed_client {
            info!("Embedding {} texts", texts.len());
            embed_client
                .embed_strings(texts_ref)
                .await
                .unwrap()
                .into_iter()
                .map(|embedding| Some(embedding))
                .collect()
        } else {
            info!("No embed client provided, skipping embedding");
            vec![None; texts.len()]
        };

        let paragraph_chunks = raw_chunks
            .into_iter()
            .zip(embeddings)
            .map(|(raw_chunk, embedding)| {
                raw_chunk.into_chunk(embedding, None, tdp.name.clone(), i_paragraph)
            })
            .collect::<Vec<Chunk>>();
        chunks.extend(paragraph_chunks);
    }

    chunks
}
