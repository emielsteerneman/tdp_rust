use std::collections::HashMap;

use data_access::{embed::EmbedClient, metadata::MetadataClient, vector::VectorClient};
use data_processing::{
    create_idf,
    utils::{load_all_chunks_from_tdps, load_all_tdp_jsons, process_text_to_words},
};
use data_structures::{IDF, intermediate::Chunk};
use tracing::info;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /* Assumption: A "document" in the inverse document frequency is a chunk */

    let _stdout_subscriber = tracing_subscriber::fmt::init();
    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);

    /* Step 1 : Load TDPs and Chunks */
    info!("Loading TDPs and Chunks");
    let tdps = load_all_tdp_jsons().await?;
    let mut chunks = load_all_chunks_from_tdps(&tdps).await?;
    // let mut chunks = chunks.into_iter().take(350).collect::<Vec<_>>();

    /* Step 2 : Create and store IDF */
    info!("Creating IDF");
    let texts: Vec<&str> = chunks.iter().map(|c| c.text.as_str()).collect();
    let idf_map = create_idf(&texts, &[1, 5, 10]);
    // print_idf_statistics(&idf_map);
    // TODO do I really need to clone idf_map here?
    metadata_client.store_idf(idf_map.clone()).await?;

    /* Step 3 : Create embeddings */
    info!("Creating embeddings");
    embed_chunks(&mut chunks, &*embed_client, &idf_map).await?;

    /* Step 4 : Store chunks */
    info!("Storing chunks");
    for chunk in chunks {
        vector_client.store_chunk(chunk).await?;
    }

    /* Step 5 : Start REPL */
    info!("Starting REPL");
    start_repl(&config, &*embed_client, &*vector_client, &*metadata_client).await?;

    Ok(())
}

pub async fn start_repl(
    config: &configuration::AppConfig,
    embed_client: &dyn EmbedClient,
    vector_client: &dyn VectorClient,
    metadata_client: &dyn MetadataClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Search REPL ---");
    println!("Type your query and press Enter. Type 'exit' to quit.");

    let idf_map = metadata_client
        .load_idf(
            config
                .data_access
                .vector
                .qdrant
                .as_ref()
                .unwrap()
                .run
                .clone(),
        )
        .await?;

    // print_idf_statistics(&idf_map);

    use std::io::{Write, stdin, stdout};

    loop {
        print!("\n> ");
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;

        let query = input.trim();
        if query.is_empty() {
            continue;
        }
        if query == "exit" || query == "quit" {
            break;
        }

        println!("Searching for: '{}'...", query);

        // 1. Generate dense embedding
        let dense = embed_client.embed_string(query).await?;

        // 2. Generate sparse embedding
        let sparse = embed_sparse(&query, &idf_map);
        println!("Sparse: {:?}", sparse);

        // 3. Search
        // let results = vector_client
        //     .search_chunks(Some(dense), Some(sparse), 5)
        //     .await?;
        let results = vector_client.search_chunks(None, Some(sparse), 15).await?;

        // 4. Display results
        if results.is_empty() {
            println!("No results found.");
        } else {
            for (i, (chunk, score)) in results.iter().enumerate() {
                // println!("\n[{}] Score: N/A", i + 1); // Qdrant search results from trait don't include score yet
                println!(
                    "\n[{i:2}] {score:.4} - {} - {} - {}",
                    chunk.league.name_pretty, chunk.team.name_pretty, chunk.year
                );
                println!("{}", chunk.text);
            }
        }
    }

    Ok(())
}

pub async fn embed_chunks(
    chunks: &mut [Chunk],
    embed_client: &dyn EmbedClient,
    idf_map: &IDF,
) -> Result<(), Box<dyn std::error::Error>> {
    // let texts = chunks
    //     .iter()
    //     .map(|chunk| chunk.text.clone())
    //     .collect::<Vec<String>>();
    // let dense_embeddings = embed_client.embed_strings(texts).await?;

    // for (chunk, embedding) in chunks.iter_mut().zip(dense_embeddings.into_iter()) {
    //     chunk.dense_embedding = embedding;
    // }

    for chunk in chunks {
        let sparse = embed_sparse(&chunk.text, idf_map);
        chunk.sparse_embedding = sparse;

        chunk.dense_embedding = vec![0.0; 1536];
    }

    Ok(())
}

pub fn embed_sparse(text: &str, idf_map: &IDF) -> HashMap<u32, f32> {
    let mut map = HashMap::new();

    let (ngram1, ngram2, ngram3) = process_text_to_words(text);
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

    for (word, (_, idf)) in items.into_iter().take(25) {
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
        let sparse = embed_sparse(&chunk.text, &idf_map);
        assert_eq!(sparse, HashMap::from([(0, 1.0), (1, 4.0), (2, 3.0)]));
    }
}
