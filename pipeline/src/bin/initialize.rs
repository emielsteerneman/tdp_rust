use data_access::embed::{EmbedClient, embed_sparse};
use data_processing::{
    create_idf,
    utils::{load_all_chunks_from_tdps, load_all_tdp_jsons},
};
use data_structures::{IDF, filter::Filter, intermediate::Chunk};
use tracing::info;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* Assumption: A "document" in the inverse document frequency is a chunk */

    let _stdout_subscriber = tracing_subscriber::fmt::init();
    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);

    let mut filter = Filter::default();
    filter.add_league("soccer_smallsize".try_into()?);
    filter.add_league("soccer_midsize".try_into()?);

    /* Step 1 : Load TDPs and Chunks */
    info!("Loading TDPs and Chunks");
    let tdps = load_all_tdp_jsons(&config.data_processing.tdps_json_root, Some(filter)).await?;
    let mut chunks = load_all_chunks_from_tdps(&tdps).await?;
    info!("Loaded {} tdps and {} chunks", tdps.len(), chunks.len());

    metadata_client.store_tdps(tdps.clone()).await?;

    /* Step 2 : Create and store IDF */
    info!("Creating IDF");
    let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
    let idf_map = create_idf(&texts, &[1, 5, 10]);
    // TODO do I really need to clone idf_map here?
    metadata_client.store_idf(idf_map.clone()).await?;

    /* Step 3 : Create embeddings */
    info!("Creating embeddings");
    embed_chunks(&mut chunks, Some(&*embed_client), &idf_map).await?;
    // embed_chunks(&mut chunks, None, &idf_map).await?;

    /* Step 4 : Store chunks */
    info!("Storing chunks");
    for chunk in chunks {
        vector_client.store_chunk(chunk).await?;
    }

    Ok(())
}

pub async fn embed_chunks(
    chunks: &mut [Chunk],
    embed_client: Option<&dyn EmbedClient>,
    idf_map: &IDF,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(embed_client) = embed_client {
        let texts = chunks
            .iter()
            .map(|chunk| chunk.text.clone())
            .collect::<Vec<String>>();
        let dense_embeddings = embed_client.embed_strings(texts).await?;

        for (chunk, embedding) in chunks.iter_mut().zip(dense_embeddings.into_iter()) {
            chunk.dense_embedding = embedding;
        }
    }

    for chunk in chunks {
        let sparse = embed_sparse(&chunk.text, idf_map);
        chunk.sparse_embedding = sparse;

        chunk.dense_embedding = vec![0.0; 1536];
    }

    Ok(())
}

pub fn print_idf_statistics(idf_map: &IDF) {
    let mut items = idf_map.0.clone().into_iter().collect::<Vec<_>>();
    let n_items = items.len();
    // Stupid lame weird sort needed because f32 does not implement Ord (f32 can be NaN)
    items.sort_by(|(_, (_, idf_a)), (_, (_, idf_b))| {
        idf_b
            .partial_cmp(&idf_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (word, (_, idf)) in items.into_iter().take(25) {
        println!("{word:<20}: {idf:.4}");
    }
    println!("Total amount of words: {n_items}");
}

#[cfg(test)]
mod tests {
    use crate::embed_sparse;
    use data_structures::{IDF, intermediate::Chunk};
    use std::collections::HashMap;

    #[test]
    fn test_sparse_embedding() {
        let idf_map = IDF::from([
            ("hello".to_string(), (0, 1.0)),
            ("world".to_string(), (1, 2.0)),
            ("hello world".to_string(), (2, 3.0)),
        ]);
        let chunk = Chunk {
            text: "hello world. I am world".to_string(),
            ..Default::default()
        };
        let sparse = embed_sparse(&chunk.text, &idf_map);
        assert_eq!(sparse, HashMap::from([(0, 1.0), (1, 4.0), (2, 3.0)]));
    }
}
