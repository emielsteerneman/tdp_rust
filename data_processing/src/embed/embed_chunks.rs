use data_access::embed::{EmbedClient, embed_sparse};
use data_structures::{IDF, embed_type::EmbedType, intermediate::Chunk};

pub async fn embed_chunks(
    chunks: &mut [Chunk],
    embed_client: &dyn EmbedClient,
    embed_type: EmbedType,
    idf_map: Option<&IDF>,
) -> Result<(), Box<dyn std::error::Error>> {
    if matches!(embed_type, EmbedType::DENSE | EmbedType::HYBRID) {
        let texts = chunks
            .iter()
            .map(|chunk| chunk.text.clone())
            .collect::<Vec<String>>();
        let dense_embeddings = embed_client.embed_strings(texts).await?;

        for (chunk, embedding) in chunks.iter_mut().zip(dense_embeddings.into_iter()) {
            chunk.dense_embedding = embedding;
        }
    }

    if matches!(embed_type, EmbedType::SPARSE | EmbedType::HYBRID)
        && let Some(idf_map) = idf_map
    {
        for chunk in chunks {
            let sparse = embed_sparse(&chunk.text, idf_map);
            chunk.sparse_embedding = sparse;

            chunk.dense_embedding = vec![0.0; 1536];
        }
    }

    Ok(())
}
