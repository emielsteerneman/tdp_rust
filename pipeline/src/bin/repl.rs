use std::collections::HashMap;

use data_access::{embed::EmbedClient, metadata::MetadataClient, vector::VectorClient};
use data_processing::utils::process_text_to_words;
use data_structures::IDF;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _stdout_subscriber = tracing_subscriber::fmt::init();
    let config = configuration::AppConfig::load_from_file("config.toml").unwrap();

    let embed_client = configuration::helpers::load_any_embed_client(&config);
    let vector_client = configuration::helpers::load_any_vector_client(&config).await?;
    let metadata_client = configuration::helpers::load_any_metadata_client(&config);

    start_repl(&*embed_client, &*vector_client, &*metadata_client).await?;

    Ok(())
}

pub async fn start_repl(
    embed_client: &dyn EmbedClient,
    vector_client: &dyn VectorClient,
    metadata_client: &dyn MetadataClient,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Search REPL ---");
    println!("Type your query and press Enter. Type 'exit' to quit.");

    let idf_map = metadata_client.load_idf().await?;

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
        let _dense = embed_client.embed_string(query).await?;

        // 2. Generate sparse embedding
        let sparse = embed_sparse(&query, &idf_map);
        println!("Sparse: {:?}", sparse);

        // 3. Search
        // let results = vector_client
        //     .search_chunks(Some(dense), Some(sparse), 5, None)
        //     .await?;
        let results = vector_client
            .search_chunks(None, Some(sparse), 15, None)
            .await?;

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
