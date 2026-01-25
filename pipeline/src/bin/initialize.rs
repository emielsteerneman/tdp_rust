use std::collections::HashMap;

use data_access::embed::EmbedClient;
use data_processing::{
    create_idf,
    utils::{load_all_chunks_from_tdps, load_all_tdp_jsons, process_text_to_words},
};
use data_structures::{IDF, intermediate::Chunk};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* Assumption: A "document" in the inverse document frequency is a chunk */

    let _stdout_subscriber = tracing_subscriber::fmt::init();
    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);

    /* Step 1 : Load TDPs and Chunks */
    let tdps = load_all_tdp_jsons().await?;
    let chunks = load_all_chunks_from_tdps(&tdps).await?;
    let mut chunks = chunks.into_iter().take(20).collect::<Vec<_>>();

    /* Step 2 : Create and store IDF */
    let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
    let idf_map = create_idf(&texts, &[1, 5, 10]);
    print_idf_statistics(&idf_map);
    // TODO do I really need to clone idf_map here?
    metadata_client.store_idf(idf_map.clone()).await?;

    /* Step 3 : Create embeddings */
    embed_chunks(&mut chunks, &*embed_client, &idf_map).await;
    println!("{:?}", chunks[0].dense_embedding);
    println!("{:?}", chunks[0].sparse_embedding);

    /* Step 4 : Store chunks */
    for chunk in chunks {
        vector_client.store_chunk(chunk).await?;
    }

    Ok(())
}

pub async fn start_repl() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

pub async fn embed_chunks(chunks: &mut [Chunk], embed_client: &dyn EmbedClient, idf_map: &IDF) {
    for chunk in chunks {
        let dense = embed_client.embed_string(&chunk.text).await.unwrap();
        let sparse = embed_sparse(chunk, idf_map);

        chunk.dense_embedding = dense;
        chunk.sparse_embedding = sparse;
    }
}

pub fn embed_sparse(chunk: &Chunk, idf_map: &IDF) -> HashMap<u32, f32> {
    let mut map = HashMap::new();

    let (ngram1, ngram2, ngram3) = process_text_to_words(&chunk.text);
    let iter = ngram1.iter().chain(ngram2.iter()).chain(ngram3.iter());

    for word in iter {
        if let Some((id, idf)) = idf_map.get(word) {
            *map.entry(*id).or_insert(0.0) += idf;
        }
    }

    map
}

pub fn print_idf_statistics(idf_map: &IDF) {
    let mut items = idf_map.clone().into_iter().collect::<Vec<_>>();
    let n_items = items.len();
    // Stupid lame weird sort needed because f32 does not implement Ord (f32 can be NaN)
    items.sort_by(|(_, (_, idf_a)), (_, (_, idf_b))| {
        idf_b
            .partial_cmp(&idf_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (word, (_, idf)) in items.into_iter().take(100) {
        println!("{word:<20}: {idf:.4}");
    }
    println!("Total amount of words: {n_items}");
}

#[cfg(test)]
mod tests {
    use crate::embed_sparse;
    use data_structures::intermediate::Chunk;
    use std::collections::HashMap;

    #[test]
    fn test_sparse_embedding() {
        let idf_map = HashMap::from([
            ("hello".to_string(), (0, 1.0)),
            ("world".to_string(), (1, 2.0)),
            ("hello world".to_string(), (2, 3.0)),
        ]);
        let chunk = Chunk {
            text: "hello world. I am world".to_string(),
            ..Default::default()
        };
        let sparse = embed_sparse(&chunk, &idf_map);
        assert_eq!(sparse, HashMap::from([(0, 1.0), (1, 4.0), (2, 3.0)]));
    }
}
